# INTEGRATION-MASTER

> Third-party API integration patterns, OAuth setup, payment providers, and webhook handling.

## CONTEXT

You are an AI coding assistant specializing in third-party integrations. Your role is to help implement robust, secure, and maintainable integrations with external services.

## INTEGRATION PRINCIPLES

### 1. Defense in Depth
- Never trust external data without validation
- Always handle API failures gracefully
- Implement circuit breakers for unstable services
- Log all external interactions

### 2. Security First
- Store credentials in environment variables
- Rotate API keys regularly
- Use least-privilege access tokens
- Encrypt sensitive data in transit and at rest

### 3. Reliability Patterns
- Implement retry with exponential backoff
- Use idempotency keys for critical operations
- Cache responses when appropriate
- Set reasonable timeouts

---

## OAUTH 2.0 INTEGRATION

### Standard OAuth Flow

```typescript
// lib/auth/oauth.ts
import { cookies } from 'next/headers';
import crypto from 'crypto';

interface OAuthConfig {
  clientId: string;
  clientSecret: string;
  authorizeUrl: string;
  tokenUrl: string;
  redirectUri: string;
  scopes: string[];
}

export class OAuthClient {
  constructor(private config: OAuthConfig) {}

  // Step 1: Generate authorization URL
  getAuthorizationUrl(): { url: string; state: string } {
    const state = crypto.randomBytes(32).toString('hex');
    const codeVerifier = crypto.randomBytes(32).toString('base64url');
    const codeChallenge = crypto
      .createHash('sha256')
      .update(codeVerifier)
      .digest('base64url');

    // Store state and verifier in cookie (or session)
    cookies().set('oauth_state', state, { httpOnly: true, secure: true });
    cookies().set('oauth_verifier', codeVerifier, { httpOnly: true, secure: true });

    const params = new URLSearchParams({
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      response_type: 'code',
      scope: this.config.scopes.join(' '),
      state,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    });

    return {
      url: `${this.config.authorizeUrl}?${params}`,
      state,
    };
  }

  // Step 2: Exchange code for tokens
  async exchangeCode(code: string, state: string): Promise<TokenResponse> {
    const savedState = cookies().get('oauth_state')?.value;
    const codeVerifier = cookies().get('oauth_verifier')?.value;

    if (state !== savedState) {
      throw new Error('Invalid state parameter');
    }

    if (!codeVerifier) {
      throw new Error('Missing code verifier');
    }

    const response = await fetch(this.config.tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: this.config.clientId,
        client_secret: this.config.clientSecret,
        code,
        redirect_uri: this.config.redirectUri,
        code_verifier: codeVerifier,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Token exchange failed: ${error}`);
    }

    // Clear OAuth cookies
    cookies().delete('oauth_state');
    cookies().delete('oauth_verifier');

    return response.json();
  }

  // Step 3: Refresh access token
  async refreshToken(refreshToken: string): Promise<TokenResponse> {
    const response = await fetch(this.config.tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: this.config.clientId,
        client_secret: this.config.clientSecret,
        refresh_token: refreshToken,
      }),
    });

    if (!response.ok) {
      throw new Error('Token refresh failed');
    }

    return response.json();
  }
}

interface TokenResponse {
  access_token: string;
  refresh_token?: string;
  expires_in: number;
  token_type: string;
  scope?: string;
}
```

### Provider Examples

#### GitHub OAuth
```typescript
const githubOAuth = new OAuthClient({
  clientId: process.env.GITHUB_CLIENT_ID!,
  clientSecret: process.env.GITHUB_CLIENT_SECRET!,
  authorizeUrl: 'https://github.com/login/oauth/authorize',
  tokenUrl: 'https://github.com/login/oauth/access_token',
  redirectUri: `${process.env.NEXT_PUBLIC_URL}/api/auth/callback/github`,
  scopes: ['read:user', 'user:email'],
});
```

#### Google OAuth
```typescript
const googleOAuth = new OAuthClient({
  clientId: process.env.GOOGLE_CLIENT_ID!,
  clientSecret: process.env.GOOGLE_CLIENT_SECRET!,
  authorizeUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
  tokenUrl: 'https://oauth2.googleapis.com/token',
  redirectUri: `${process.env.NEXT_PUBLIC_URL}/api/auth/callback/google`,
  scopes: ['openid', 'profile', 'email'],
});
```

---

## PAYMENT PROVIDER INTEGRATION

### Stripe

```typescript
// lib/stripe/client.ts
import Stripe from 'stripe';

export const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2023-10-16',
  typescript: true,
});

// Create payment intent
export async function createPaymentIntent(
  amount: number,
  currency: string = 'usd',
  metadata?: Record<string, string>
) {
  return stripe.paymentIntents.create({
    amount: Math.round(amount * 100), // Convert to cents
    currency,
    metadata,
    automatic_payment_methods: { enabled: true },
  });
}

// Create customer
export async function createCustomer(email: string, name?: string) {
  return stripe.customers.create({ email, name });
}

// Create subscription
export async function createSubscription(
  customerId: string,
  priceId: string,
  trialDays?: number
) {
  return stripe.subscriptions.create({
    customer: customerId,
    items: [{ price: priceId }],
    trial_period_days: trialDays,
    payment_behavior: 'default_incomplete',
    expand: ['latest_invoice.payment_intent'],
  });
}

// Cancel subscription
export async function cancelSubscription(subscriptionId: string) {
  return stripe.subscriptions.cancel(subscriptionId);
}
```

### Stripe Webhook Handler
```typescript
// app/api/webhooks/stripe/route.ts
import { headers } from 'next/headers';
import { stripe } from '@/lib/stripe/client';
import { updateSubscription, handlePaymentFailed } from '@/lib/db/subscriptions';

export async function POST(request: Request) {
  const body = await request.text();
  const signature = headers().get('stripe-signature')!;

  let event: Stripe.Event;

  try {
    event = stripe.webhooks.constructEvent(
      body,
      signature,
      process.env.STRIPE_WEBHOOK_SECRET!
    );
  } catch (err) {
    console.error('Webhook verification failed:', err);
    return new Response('Webhook Error', { status: 400 });
  }

  // Handle different event types
  const handlers: Record<string, (data: any) => Promise<void>> = {
    'customer.subscription.created': async (subscription) => {
      await updateSubscription(subscription.id, {
        status: subscription.status,
        currentPeriodEnd: new Date(subscription.current_period_end * 1000),
      });
    },
    
    'customer.subscription.updated': async (subscription) => {
      await updateSubscription(subscription.id, {
        status: subscription.status,
        currentPeriodEnd: new Date(subscription.current_period_end * 1000),
        cancelAtPeriodEnd: subscription.cancel_at_period_end,
      });
    },
    
    'customer.subscription.deleted': async (subscription) => {
      await updateSubscription(subscription.id, { status: 'canceled' });
    },
    
    'invoice.payment_failed': async (invoice) => {
      await handlePaymentFailed(invoice.subscription as string);
    },
  };

  const handler = handlers[event.type];
  if (handler) {
    try {
      await handler(event.data.object);
    } catch (err) {
      console.error(`Error handling ${event.type}:`, err);
      return new Response('Handler Error', { status: 500 });
    }
  }

  return new Response('OK', { status: 200 });
}
```

---

## API CLIENT PATTERNS

### Base API Client with Retry

```typescript
// lib/api/client.ts
interface ApiClientConfig {
  baseUrl: string;
  apiKey?: string;
  timeout?: number;
  maxRetries?: number;
}

export class ApiClient {
  private baseUrl: string;
  private headers: Record<string, string>;
  private timeout: number;
  private maxRetries: number;

  constructor(config: ApiClientConfig) {
    this.baseUrl = config.baseUrl;
    this.timeout = config.timeout ?? 30000;
    this.maxRetries = config.maxRetries ?? 3;
    this.headers = {
      'Content-Type': 'application/json',
    };

    if (config.apiKey) {
      this.headers['Authorization'] = `Bearer ${config.apiKey}`;
    }
  }

  async request<T>(
    method: string,
    path: string,
    options?: {
      body?: unknown;
      params?: Record<string, string>;
      headers?: Record<string, string>;
    }
  ): Promise<T> {
    const url = new URL(path, this.baseUrl);
    
    if (options?.params) {
      Object.entries(options.params).forEach(([key, value]) => {
        url.searchParams.set(key, value);
      });
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      try {
        const response = await fetch(url.toString(), {
          method,
          headers: { ...this.headers, ...options?.headers },
          body: options?.body ? JSON.stringify(options.body) : undefined,
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
          // Don't retry client errors (4xx)
          if (response.status >= 400 && response.status < 500) {
            const error = await response.json().catch(() => ({}));
            throw new ApiError(response.status, error.message || 'Client error');
          }
          
          // Retry server errors (5xx)
          throw new ApiError(response.status, 'Server error');
        }

        return response.json();
      } catch (error) {
        lastError = error as Error;

        if (error instanceof ApiError && error.status < 500) {
          throw error; // Don't retry client errors
        }

        if (attempt < this.maxRetries) {
          // Exponential backoff
          await this.delay(Math.pow(2, attempt) * 1000);
        }
      }
    }

    throw lastError;
  }

  async get<T>(path: string, params?: Record<string, string>): Promise<T> {
    return this.request<T>('GET', path, { params });
  }

  async post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('POST', path, { body });
  }

  async put<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PUT', path, { body });
  }

  async delete<T>(path: string): Promise<T> {
    return this.request<T>('DELETE', path);
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}
```

### Circuit Breaker Pattern

```typescript
// lib/api/circuit-breaker.ts
enum CircuitState {
  CLOSED = 'CLOSED',
  OPEN = 'OPEN',
  HALF_OPEN = 'HALF_OPEN',
}

interface CircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeout: number;
  monitoringWindow: number;
}

export class CircuitBreaker {
  private state: CircuitState = CircuitState.CLOSED;
  private failures: number[] = [];
  private lastFailureTime: number = 0;

  constructor(
    private config: CircuitBreakerConfig = {
      failureThreshold: 5,
      recoveryTimeout: 30000,
      monitoringWindow: 60000,
    }
  ) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === CircuitState.OPEN) {
      if (Date.now() - this.lastFailureTime > this.config.recoveryTimeout) {
        this.state = CircuitState.HALF_OPEN;
      } else {
        throw new Error('Circuit breaker is OPEN');
      }
    }

    try {
      const result = await operation();
      
      if (this.state === CircuitState.HALF_OPEN) {
        this.state = CircuitState.CLOSED;
        this.failures = [];
      }
      
      return result;
    } catch (error) {
      this.recordFailure();
      throw error;
    }
  }

  private recordFailure() {
    const now = Date.now();
    this.lastFailureTime = now;
    
    // Clean old failures outside monitoring window
    this.failures = this.failures.filter(
      (time) => now - time < this.config.monitoringWindow
    );
    
    this.failures.push(now);

    if (this.failures.length >= this.config.failureThreshold) {
      this.state = CircuitState.OPEN;
    }
  }

  getState(): CircuitState {
    return this.state;
  }
}
```

---

## WEBHOOK HANDLING

### Generic Webhook Handler

```typescript
// lib/webhooks/handler.ts
import crypto from 'crypto';

export interface WebhookConfig {
  secret: string;
  signatureHeader: string;
  signatureAlgorithm: 'sha256' | 'sha1';
  timestampHeader?: string;
  timestampTolerance?: number; // seconds
}

export class WebhookHandler {
  constructor(private config: WebhookConfig) {}

  verifySignature(
    payload: string | Buffer,
    headers: Record<string, string>
  ): boolean {
    const signature = headers[this.config.signatureHeader.toLowerCase()];
    
    if (!signature) {
      throw new Error('Missing signature header');
    }

    // Verify timestamp if configured
    if (this.config.timestampHeader && this.config.timestampTolerance) {
      const timestamp = headers[this.config.timestampHeader.toLowerCase()];
      if (!this.verifyTimestamp(timestamp)) {
        throw new Error('Webhook timestamp is too old');
      }
    }

    const expectedSignature = this.computeSignature(payload);
    return crypto.timingSafeEqual(
      Buffer.from(signature),
      Buffer.from(expectedSignature)
    );
  }

  private computeSignature(payload: string | Buffer): string {
    const hmac = crypto.createHmac(
      this.config.signatureAlgorithm,
      this.config.secret
    );
    hmac.update(payload);
    return hmac.digest('hex');
  }

  private verifyTimestamp(timestamp: string): boolean {
    const webhookTime = parseInt(timestamp, 10) * 1000;
    const now = Date.now();
    const tolerance = (this.config.timestampTolerance || 300) * 1000;
    return Math.abs(now - webhookTime) < tolerance;
  }
}
```

### Webhook Queue (for reliability)

```typescript
// lib/webhooks/queue.ts
interface QueuedWebhook {
  id: string;
  payload: unknown;
  eventType: string;
  receivedAt: Date;
  processedAt?: Date;
  attempts: number;
  lastError?: string;
}

export class WebhookQueue {
  async enqueue(eventType: string, payload: unknown): Promise<string> {
    const id = crypto.randomUUID();
    
    // Store in database
    await db.webhookQueue.create({
      id,
      eventType,
      payload: JSON.stringify(payload),
      receivedAt: new Date(),
      attempts: 0,
    });

    // Trigger async processing
    this.processAsync(id);
    
    return id;
  }

  private async processAsync(id: string) {
    // Use a job queue in production (Bull, BullMQ, etc.)
    setTimeout(() => this.process(id), 0);
  }

  private async process(id: string) {
    const webhook = await db.webhookQueue.findUnique({ where: { id } });
    
    if (!webhook || webhook.processedAt) return;

    try {
      await this.handleEvent(webhook.eventType, JSON.parse(webhook.payload));
      
      await db.webhookQueue.update({
        where: { id },
        data: { processedAt: new Date() },
      });
    } catch (error) {
      await db.webhookQueue.update({
        where: { id },
        data: {
          attempts: webhook.attempts + 1,
          lastError: error.message,
        },
      });

      // Retry with backoff
      if (webhook.attempts < 3) {
        setTimeout(() => this.process(id), Math.pow(2, webhook.attempts) * 1000);
      }
    }
  }

  private async handleEvent(eventType: string, payload: unknown) {
    const handlers: Record<string, (payload: any) => Promise<void>> = {
      'order.created': handleOrderCreated,
      'payment.completed': handlePaymentCompleted,
      // ... more handlers
    };

    const handler = handlers[eventType];
    if (handler) {
      await handler(payload);
    }
  }
}
```

---

## COMMON INTEGRATIONS

### Email (SendGrid/Resend)

```typescript
// lib/email/client.ts
interface EmailConfig {
  from: string;
  replyTo?: string;
}

interface SendEmailOptions {
  to: string | string[];
  subject: string;
  html: string;
  text?: string;
}

// Using Resend
import { Resend } from 'resend';

const resend = new Resend(process.env.RESEND_API_KEY);

export async function sendEmail(options: SendEmailOptions) {
  const { data, error } = await resend.emails.send({
    from: 'Your App <noreply@yourapp.com>',
    to: options.to,
    subject: options.subject,
    html: options.html,
    text: options.text,
  });

  if (error) {
    throw new Error(`Failed to send email: ${error.message}`);
  }

  return data;
}
```

### File Storage (S3/Cloudflare R2)

```typescript
// lib/storage/s3.ts
import { S3Client, PutObjectCommand, GetObjectCommand } from '@aws-sdk/client-s3';
import { getSignedUrl } from '@aws-sdk/s3-request-presigner';

const s3 = new S3Client({
  region: process.env.AWS_REGION || 'auto',
  endpoint: process.env.S3_ENDPOINT, // For R2/MinIO
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
  },
});

export async function uploadFile(
  key: string,
  body: Buffer | Blob,
  contentType: string
): Promise<string> {
  await s3.send(
    new PutObjectCommand({
      Bucket: process.env.S3_BUCKET!,
      Key: key,
      Body: body,
      ContentType: contentType,
    })
  );

  return `${process.env.S3_PUBLIC_URL}/${key}`;
}

export async function getSignedUploadUrl(
  key: string,
  contentType: string,
  expiresIn: number = 3600
): Promise<string> {
  const command = new PutObjectCommand({
    Bucket: process.env.S3_BUCKET!,
    Key: key,
    ContentType: contentType,
  });

  return getSignedUrl(s3, command, { expiresIn });
}
```

### Analytics (Posthog/Mixpanel)

```typescript
// lib/analytics/client.ts
import { PostHog } from 'posthog-node';

const posthog = new PostHog(process.env.POSTHOG_API_KEY!, {
  host: process.env.POSTHOG_HOST || 'https://app.posthog.com',
});

export function trackEvent(
  userId: string,
  event: string,
  properties?: Record<string, unknown>
) {
  posthog.capture({
    distinctId: userId,
    event,
    properties,
  });
}

export function identifyUser(
  userId: string,
  properties: Record<string, unknown>
) {
  posthog.identify({
    distinctId: userId,
    properties,
  });
}

// Flush on shutdown
process.on('beforeExit', async () => {
  await posthog.shutdown();
});
```

---

## INTEGRATION CHECKLIST

### Before Integration
- [ ] Review API documentation thoroughly
- [ ] Understand rate limits and quotas
- [ ] Identify required scopes/permissions
- [ ] Plan error handling strategy
- [ ] Design idempotency approach

### During Implementation
- [ ] Store credentials in environment variables
- [ ] Implement request/response logging
- [ ] Add retry logic with backoff
- [ ] Handle all error cases
- [ ] Validate incoming data

### Testing
- [ ] Test with API sandbox/test mode
- [ ] Mock external calls in unit tests
- [ ] Test error scenarios
- [ ] Verify webhook signature handling
- [ ] Load test for rate limits

### Monitoring
- [ ] Track API response times
- [ ] Monitor error rates
- [ ] Alert on integration failures
- [ ] Log all webhook events
- [ ] Dashboard for integration health
