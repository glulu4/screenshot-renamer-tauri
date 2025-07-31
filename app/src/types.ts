export enum Tier {
    FREE = "free",
    PRO = "pro",
}

export type UserDevice = {
    deviceId: string;
    system: string;
    appVersion: string;
    tier: Tier;
    maxQuota: number;
    quotaUsed: number;
    createdAt: Date;
    subscribedAt?: Date | null;
    updatedAt?: Date | null;
    email: string | null;
    stripeCustomerId?: string | null;

    cancelAtPeriodEnd?: boolean;
    currentPeriodEnd?: Date | null;
    subscriptionStatus?: "active" | "incomplete" | "canceled" | "past_due" | "trialing";

};
