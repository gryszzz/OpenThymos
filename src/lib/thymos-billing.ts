/**
 * Thymos Stripe Billing integration.
 *
 * After a run completes, report usage to Stripe based on the budget consumed.
 * Works with Stripe's usage-based billing (metered subscriptions) or
 * usage records API.
 *
 * Setup:
 *   1. Create a metered price in Stripe with `usage_type: metered`
 *   2. Set STRIPE_SECRET_KEY and STRIPE_METERED_PRICE_ID env vars
 *   3. Call `reportRunUsage()` after each completed run
 */

const STRIPE_API = "https://api.stripe.com/v1";

interface RunUsage {
  /** Stripe subscription item ID for the user's metered plan. */
  subscriptionItemId: string;
  /** Budget consumed by the run (from AgentRunSummary). */
  budgetUsed: {
    tokens: number;
    toolCalls: number;
    wallClockMs: number;
    usdMillicents: number;
  };
  /** Run ID for idempotency. */
  runId: string;
}

/**
 * Report run usage to Stripe as a usage record on a metered subscription.
 * The quantity is in millicents (1/1000 of a cent).
 */
export async function reportRunUsage(usage: RunUsage): Promise<void> {
  const apiKey = process.env.STRIPE_SECRET_KEY;
  if (!apiKey) {
    console.warn("STRIPE_SECRET_KEY not set, skipping usage report");
    return;
  }

  // Calculate total cost in usage units.
  // You can customize the pricing model here.
  const quantity = Math.max(1, usage.budgetUsed.usdMillicents);

  const params = new URLSearchParams({
    quantity: String(quantity),
    timestamp: String(Math.floor(Date.now() / 1000)),
    action: "increment",
  });

  const res = await fetch(
    `${STRIPE_API}/subscription_items/${usage.subscriptionItemId}/usage_records`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/x-www-form-urlencoded",
        "Idempotency-Key": `thymos-run-${usage.runId}`,
      },
      body: params.toString(),
    },
  );

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Stripe usage report failed (${res.status}): ${body}`);
  }
}

/**
 * Look up a customer's metered subscription item ID.
 * Call this once during session setup and cache the result.
 */
export async function findMeteredSubscriptionItem(
  customerId: string,
  priceId?: string,
): Promise<string | null> {
  const apiKey = process.env.STRIPE_SECRET_KEY;
  if (!apiKey) return null;

  const targetPrice = priceId ?? process.env.STRIPE_METERED_PRICE_ID;
  if (!targetPrice) return null;

  const res = await fetch(
    `${STRIPE_API}/subscriptions?customer=${customerId}&status=active`,
    {
      headers: { Authorization: `Bearer ${apiKey}` },
    },
  );

  if (!res.ok) return null;

  const data = (await res.json()) as {
    data: Array<{
      items: {
        data: Array<{
          id: string;
          price: { id: string };
        }>;
      };
    }>;
  };

  for (const sub of data.data) {
    for (const item of sub.items.data) {
      if (item.price.id === targetPrice) {
        return item.id;
      }
    }
  }
  return null;
}
