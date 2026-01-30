# E-COMMERCE Project Template

> Comprehensive e-commerce project structure with product catalog, cart, checkout, and admin.

## Architecture Overview

```
src/
├── app/                           # App Router (Next.js 14+)
│   ├── (shop)/                    # Public shopping routes
│   │   ├── page.tsx               # Homepage
│   │   ├── products/              # Product pages
│   │   │   ├── page.tsx           # Catalog listing
│   │   │   └── [slug]/page.tsx    # Product detail
│   │   ├── cart/page.tsx          # Shopping cart
│   │   ├── checkout/              # Checkout flow
│   │   │   ├── page.tsx           # Checkout page
│   │   │   ├── success/page.tsx   # Order confirmation
│   │   │   └── cancel/page.tsx    # Payment cancelled
│   │   └── orders/                # Order history
│   │       └── [id]/page.tsx      # Order detail
│   ├── (auth)/                    # Auth routes
│   │   ├── login/page.tsx
│   │   ├── register/page.tsx
│   │   └── account/page.tsx       # User account
│   ├── admin/                     # Admin dashboard
│   │   ├── layout.tsx             # Admin layout with sidebar
│   │   ├── page.tsx               # Dashboard overview
│   │   ├── products/              # Product management
│   │   │   ├── page.tsx           # Product list
│   │   │   ├── new/page.tsx       # Add product
│   │   │   └── [id]/edit/page.tsx # Edit product
│   │   ├── orders/                # Order management
│   │   │   ├── page.tsx           # Order list
│   │   │   └── [id]/page.tsx      # Order detail
│   │   ├── customers/page.tsx     # Customer list
│   │   └── settings/page.tsx      # Store settings
│   └── api/                       # API routes
│       ├── products/
│       ├── cart/
│       ├── checkout/
│       ├── orders/
│       └── webhooks/
│           └── stripe/route.ts    # Stripe webhooks
├── components/
│   ├── ui/                        # Base UI components
│   ├── shop/                      # Shop-specific
│   │   ├── ProductCard.tsx
│   │   ├── ProductGrid.tsx
│   │   ├── ProductGallery.tsx
│   │   ├── AddToCartButton.tsx
│   │   ├── QuantitySelector.tsx
│   │   └── PriceDisplay.tsx
│   ├── cart/
│   │   ├── CartDrawer.tsx
│   │   ├── CartItem.tsx
│   │   ├── CartSummary.tsx
│   │   └── CartProvider.tsx
│   ├── checkout/
│   │   ├── CheckoutForm.tsx
│   │   ├── ShippingForm.tsx
│   │   ├── PaymentForm.tsx
│   │   └── OrderSummary.tsx
│   └── admin/
│       ├── Sidebar.tsx
│       ├── StatsCard.tsx
│       ├── DataTable.tsx
│       └── ProductForm.tsx
├── lib/
│   ├── db/                        # Database
│   │   ├── schema.ts              # Drizzle/Prisma schema
│   │   └── queries/
│   │       ├── products.ts
│   │       ├── orders.ts
│   │       └── customers.ts
│   ├── stripe/                    # Stripe integration
│   │   ├── client.ts
│   │   ├── checkout.ts
│   │   └── webhooks.ts
│   ├── cart/                      # Cart logic
│   │   ├── actions.ts
│   │   └── utils.ts
│   └── auth/                      # Authentication
│       └── config.ts
├── hooks/
│   ├── useCart.ts
│   ├── useProduct.ts
│   └── useCheckout.ts
└── types/
    ├── product.ts
    ├── cart.ts
    ├── order.ts
    └── customer.ts
```

## Database Schema

### Products Table
```sql
CREATE TABLE products (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL,
  slug VARCHAR(255) UNIQUE NOT NULL,
  description TEXT,
  price DECIMAL(10,2) NOT NULL,
  compare_at_price DECIMAL(10,2),      -- Original price for sales
  cost_per_item DECIMAL(10,2),         -- Cost for profit calculation
  sku VARCHAR(100) UNIQUE,
  barcode VARCHAR(100),
  track_inventory BOOLEAN DEFAULT true,
  inventory_quantity INTEGER DEFAULT 0,
  weight DECIMAL(10,2),                 -- For shipping
  status VARCHAR(20) DEFAULT 'draft',   -- draft, active, archived
  category_id UUID REFERENCES categories(id),
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE product_images (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID REFERENCES products(id) ON DELETE CASCADE,
  url VARCHAR(500) NOT NULL,
  alt_text VARCHAR(255),
  position INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE product_variants (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID REFERENCES products(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,           -- "Size: Large, Color: Blue"
  sku VARCHAR(100) UNIQUE,
  price DECIMAL(10,2) NOT NULL,
  inventory_quantity INTEGER DEFAULT 0,
  options JSONB,                        -- {"size": "L", "color": "blue"}
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Orders Table
```sql
CREATE TABLE orders (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_number VARCHAR(50) UNIQUE NOT NULL,
  customer_id UUID REFERENCES customers(id),
  email VARCHAR(255) NOT NULL,
  
  -- Status
  status VARCHAR(30) DEFAULT 'pending',  -- pending, processing, shipped, delivered, cancelled
  payment_status VARCHAR(30) DEFAULT 'pending',  -- pending, paid, failed, refunded
  fulfillment_status VARCHAR(30) DEFAULT 'unfulfilled',
  
  -- Totals
  subtotal DECIMAL(10,2) NOT NULL,
  shipping_total DECIMAL(10,2) DEFAULT 0,
  tax_total DECIMAL(10,2) DEFAULT 0,
  discount_total DECIMAL(10,2) DEFAULT 0,
  grand_total DECIMAL(10,2) NOT NULL,
  
  -- Addresses (JSONB for flexibility)
  shipping_address JSONB,
  billing_address JSONB,
  
  -- Payment
  stripe_payment_intent_id VARCHAR(255),
  stripe_checkout_session_id VARCHAR(255),
  
  -- Timestamps
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  shipped_at TIMESTAMP,
  delivered_at TIMESTAMP
);

CREATE TABLE order_items (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
  product_id UUID REFERENCES products(id),
  variant_id UUID REFERENCES product_variants(id),
  name VARCHAR(255) NOT NULL,           -- Snapshot of product name
  sku VARCHAR(100),
  price DECIMAL(10,2) NOT NULL,         -- Price at time of order
  quantity INTEGER NOT NULL,
  total DECIMAL(10,2) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Customers Table
```sql
CREATE TABLE customers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  first_name VARCHAR(100),
  last_name VARCHAR(100),
  phone VARCHAR(50),
  accepts_marketing BOOLEAN DEFAULT false,
  stripe_customer_id VARCHAR(255),
  total_spent DECIMAL(10,2) DEFAULT 0,
  order_count INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE customer_addresses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  customer_id UUID REFERENCES customers(id) ON DELETE CASCADE,
  is_default BOOLEAN DEFAULT false,
  first_name VARCHAR(100),
  last_name VARCHAR(100),
  address_line1 VARCHAR(255) NOT NULL,
  address_line2 VARCHAR(255),
  city VARCHAR(100) NOT NULL,
  state VARCHAR(100),
  postal_code VARCHAR(20) NOT NULL,
  country VARCHAR(2) NOT NULL,          -- ISO country code
  phone VARCHAR(50),
  created_at TIMESTAMP DEFAULT NOW()
);
```

## Stripe Integration

### Checkout Session
```typescript
// lib/stripe/checkout.ts
import Stripe from 'stripe';
import { CartItem } from '@/types/cart';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

export async function createCheckoutSession(
  items: CartItem[],
  customerId?: string
) {
  const lineItems: Stripe.Checkout.SessionCreateParams.LineItem[] = items.map(
    (item) => ({
      price_data: {
        currency: 'usd',
        product_data: {
          name: item.name,
          images: item.image ? [item.image] : undefined,
        },
        unit_amount: Math.round(item.price * 100), // Stripe uses cents
      },
      quantity: item.quantity,
    })
  );

  const session = await stripe.checkout.sessions.create({
    mode: 'payment',
    line_items: lineItems,
    success_url: `${process.env.NEXT_PUBLIC_URL}/checkout/success?session_id={CHECKOUT_SESSION_ID}`,
    cancel_url: `${process.env.NEXT_PUBLIC_URL}/checkout/cancel`,
    customer: customerId,
    shipping_address_collection: {
      allowed_countries: ['US', 'CA', 'GB', 'AU'],
    },
    billing_address_collection: 'required',
    metadata: {
      // Store cart ID or other reference
    },
  });

  return session;
}
```

### Webhook Handler
```typescript
// app/api/webhooks/stripe/route.ts
import { headers } from 'next/headers';
import Stripe from 'stripe';
import { createOrder, updateOrderStatus } from '@/lib/db/queries/orders';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);
const webhookSecret = process.env.STRIPE_WEBHOOK_SECRET!;

export async function POST(request: Request) {
  const body = await request.text();
  const signature = headers().get('stripe-signature')!;

  let event: Stripe.Event;

  try {
    event = stripe.webhooks.constructEvent(body, signature, webhookSecret);
  } catch (err) {
    console.error('Webhook signature verification failed');
    return new Response('Webhook Error', { status: 400 });
  }

  switch (event.type) {
    case 'checkout.session.completed': {
      const session = event.data.object as Stripe.Checkout.Session;
      await handleCheckoutComplete(session);
      break;
    }
    
    case 'payment_intent.succeeded': {
      const paymentIntent = event.data.object as Stripe.PaymentIntent;
      await updateOrderStatus(paymentIntent.id, 'paid');
      break;
    }
    
    case 'payment_intent.payment_failed': {
      const paymentIntent = event.data.object as Stripe.PaymentIntent;
      await updateOrderStatus(paymentIntent.id, 'failed');
      break;
    }
    
    case 'charge.refunded': {
      const charge = event.data.object as Stripe.Charge;
      await handleRefund(charge);
      break;
    }
  }

  return new Response('OK', { status: 200 });
}

async function handleCheckoutComplete(session: Stripe.Checkout.Session) {
  // Create order from checkout session
  const order = await createOrder({
    stripeCheckoutSessionId: session.id,
    stripePaymentIntentId: session.payment_intent as string,
    email: session.customer_details?.email!,
    subtotal: session.amount_subtotal! / 100,
    total: session.amount_total! / 100,
    shippingAddress: session.shipping_details?.address,
    paymentStatus: 'paid',
  });
  
  // Send confirmation email
  // Update inventory
  // Notify admin
}
```

## Cart Management

### Cart Context
```typescript
// components/cart/CartProvider.tsx
'use client';

import { createContext, useContext, useReducer, useEffect } from 'react';
import { CartItem, CartState } from '@/types/cart';

type CartAction =
  | { type: 'ADD_ITEM'; payload: CartItem }
  | { type: 'REMOVE_ITEM'; payload: string }
  | { type: 'UPDATE_QUANTITY'; payload: { id: string; quantity: number } }
  | { type: 'CLEAR_CART' }
  | { type: 'LOAD_CART'; payload: CartItem[] };

const CartContext = createContext<{
  state: CartState;
  addItem: (item: CartItem) => void;
  removeItem: (id: string) => void;
  updateQuantity: (id: string, quantity: number) => void;
  clearCart: () => void;
} | null>(null);

function cartReducer(state: CartState, action: CartAction): CartState {
  switch (action.type) {
    case 'ADD_ITEM': {
      const existingIndex = state.items.findIndex(
        (item) => item.id === action.payload.id
      );
      
      if (existingIndex > -1) {
        const newItems = [...state.items];
        newItems[existingIndex].quantity += action.payload.quantity;
        return { ...state, items: newItems };
      }
      
      return { ...state, items: [...state.items, action.payload] };
    }
    
    case 'REMOVE_ITEM':
      return {
        ...state,
        items: state.items.filter((item) => item.id !== action.payload),
      };
    
    case 'UPDATE_QUANTITY': {
      const newItems = state.items.map((item) =>
        item.id === action.payload.id
          ? { ...item, quantity: action.payload.quantity }
          : item
      );
      return { ...state, items: newItems };
    }
    
    case 'CLEAR_CART':
      return { ...state, items: [] };
    
    case 'LOAD_CART':
      return { ...state, items: action.payload };
    
    default:
      return state;
  }
}

export function CartProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(cartReducer, { items: [] });

  // Persist to localStorage
  useEffect(() => {
    const saved = localStorage.getItem('cart');
    if (saved) {
      dispatch({ type: 'LOAD_CART', payload: JSON.parse(saved) });
    }
  }, []);

  useEffect(() => {
    localStorage.setItem('cart', JSON.stringify(state.items));
  }, [state.items]);

  const addItem = (item: CartItem) => dispatch({ type: 'ADD_ITEM', payload: item });
  const removeItem = (id: string) => dispatch({ type: 'REMOVE_ITEM', payload: id });
  const updateQuantity = (id: string, quantity: number) =>
    dispatch({ type: 'UPDATE_QUANTITY', payload: { id, quantity } });
  const clearCart = () => dispatch({ type: 'CLEAR_CART' });

  return (
    <CartContext.Provider value={{ state, addItem, removeItem, updateQuantity, clearCart }}>
      {children}
    </CartContext.Provider>
  );
}

export function useCart() {
  const context = useContext(CartContext);
  if (!context) {
    throw new Error('useCart must be used within CartProvider');
  }
  return context;
}
```

## Admin Dashboard Components

### Stats Overview
```typescript
// components/admin/StatsCard.tsx
interface StatsCardProps {
  title: string;
  value: string | number;
  change?: number;
  icon: React.ReactNode;
}

export function StatsCard({ title, value, change, icon }: StatsCardProps) {
  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-500">{title}</p>
          <p className="text-2xl font-bold mt-1">{value}</p>
          {change !== undefined && (
            <p className={`text-sm mt-1 ${change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              {change >= 0 ? '+' : ''}{change}% from last month
            </p>
          )}
        </div>
        <div className="p-3 bg-blue-50 rounded-full">
          {icon}
        </div>
      </div>
    </div>
  );
}
```

### Product Form
```typescript
// components/admin/ProductForm.tsx
'use client';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const productSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  slug: z.string().min(1, 'Slug is required'),
  description: z.string().optional(),
  price: z.number().min(0, 'Price must be positive'),
  compareAtPrice: z.number().optional(),
  sku: z.string().optional(),
  trackInventory: z.boolean().default(true),
  inventoryQuantity: z.number().int().min(0).default(0),
  status: z.enum(['draft', 'active', 'archived']).default('draft'),
  categoryId: z.string().optional(),
});

type ProductFormData = z.infer<typeof productSchema>;

export function ProductForm({
  product,
  onSubmit,
}: {
  product?: ProductFormData;
  onSubmit: (data: ProductFormData) => void;
}) {
  const {
    register,
    handleSubmit,
    watch,
    formState: { errors, isSubmitting },
  } = useForm<ProductFormData>({
    resolver: zodResolver(productSchema),
    defaultValues: product,
  });

  const trackInventory = watch('trackInventory');

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
      <div className="grid grid-cols-2 gap-6">
        <div>
          <label className="block text-sm font-medium">Product Name</label>
          <input
            {...register('name')}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
          />
          {errors.name && (
            <p className="text-red-500 text-sm mt-1">{errors.name.message}</p>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium">URL Slug</label>
          <input
            {...register('slug')}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
          />
          {errors.slug && (
            <p className="text-red-500 text-sm mt-1">{errors.slug.message}</p>
          )}
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium">Description</label>
        <textarea
          {...register('description')}
          rows={4}
          className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
        />
      </div>

      <div className="grid grid-cols-3 gap-6">
        <div>
          <label className="block text-sm font-medium">Price</label>
          <input
            type="number"
            step="0.01"
            {...register('price', { valueAsNumber: true })}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
          />
        </div>

        <div>
          <label className="block text-sm font-medium">Compare at Price</label>
          <input
            type="number"
            step="0.01"
            {...register('compareAtPrice', { valueAsNumber: true })}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
          />
        </div>

        <div>
          <label className="block text-sm font-medium">SKU</label>
          <input
            {...register('sku')}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
          />
        </div>
      </div>

      <div className="space-y-4">
        <label className="flex items-center">
          <input type="checkbox" {...register('trackInventory')} className="mr-2" />
          <span className="text-sm">Track inventory</span>
        </label>

        {trackInventory && (
          <div>
            <label className="block text-sm font-medium">Inventory Quantity</label>
            <input
              type="number"
              {...register('inventoryQuantity', { valueAsNumber: true })}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
            />
          </div>
        )}
      </div>

      <div>
        <label className="block text-sm font-medium">Status</label>
        <select
          {...register('status')}
          className="mt-1 block w-full rounded-md border-gray-300 shadow-sm"
        >
          <option value="draft">Draft</option>
          <option value="active">Active</option>
          <option value="archived">Archived</option>
        </select>
      </div>

      <button
        type="submit"
        disabled={isSubmitting}
        className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
      >
        {isSubmitting ? 'Saving...' : 'Save Product'}
      </button>
    </form>
  );
}
```

## Environment Variables

```env
# Database
DATABASE_URL=postgresql://...

# Stripe
STRIPE_SECRET_KEY=sk_...
STRIPE_PUBLISHABLE_KEY=pk_...
STRIPE_WEBHOOK_SECRET=whsec_...

# Auth
NEXTAUTH_SECRET=...
NEXTAUTH_URL=http://localhost:3000

# App
NEXT_PUBLIC_URL=http://localhost:3000
NEXT_PUBLIC_APP_NAME="My Store"
```

## Key Features Checklist

### Product Catalog
- [ ] Product listing with filters (category, price, sort)
- [ ] Product detail page with gallery
- [ ] Product variants (size, color, etc.)
- [ ] Related products
- [ ] Search functionality
- [ ] Category navigation

### Shopping Cart
- [ ] Add/remove/update items
- [ ] Persistent cart (localStorage + server sync)
- [ ] Cart drawer/sidebar
- [ ] Quantity validation against inventory
- [ ] Price calculations with discounts

### Checkout
- [ ] Guest checkout
- [ ] User checkout (saved addresses)
- [ ] Shipping options
- [ ] Discount codes
- [ ] Tax calculation
- [ ] Stripe payment integration
- [ ] Order confirmation page
- [ ] Email notifications

### User Account
- [ ] Registration/Login
- [ ] Order history
- [ ] Saved addresses
- [ ] Wishlist
- [ ] Account settings

### Admin Dashboard
- [ ] Dashboard overview (sales, orders, customers)
- [ ] Product management (CRUD)
- [ ] Order management (view, update status, refund)
- [ ] Customer management
- [ ] Discount/coupon management
- [ ] Analytics and reports
- [ ] Store settings

### Technical
- [ ] Responsive design
- [ ] SEO optimization (meta tags, structured data)
- [ ] Image optimization
- [ ] Inventory management
- [ ] Webhook handling
- [ ] Error handling
- [ ] Loading states
