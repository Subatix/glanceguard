/** User-facing copy for glance / privacy alerts. Keep jargon-free — no scores, embeddings, or "observer". */

export const alertHeadline = "Someone else may see your screen";

/** macOS notification body (standard + compact styles). */
export const alertDetailForNotification =
  "We think someone besides you might be paying attention to your screen. If it's safe to, look around.";

/** Shorter line under the full-screen alert overlay. */
export const alertOverlaySupporting =
  "When you can, take a quick glance around—you are still in charge here.";

export const alertNotificationTitleNative = alertHeadline;

export const alertNotificationTitleCompact = "Heads-up";

export const monitoringProtectionCopy = {
  idle: {
    title: "Protection is off",
    body: "Turn on monitoring when you want a friendly nudge if someone else seems to notice your screen.",
  },
  monitoring: {
    title: "Protection is on",
    body: "GlanceGuard is looking out for you. Your video stays on this Mac—not sent anywhere.",
  },
  alert: {
    title: alertHeadline,
    body: alertDetailForNotification,
  },
  cooldown: {
    title: "Short pause",
    body: "We just tapped you once. We'll wait a moment before another heads-up.",
  },
} as const satisfies Record<
  "idle" | "monitoring" | "alert" | "cooldown",
  { readonly title: string; readonly body: string }
>;

/** Brief labels for header / status chips — plain language without developer jargon. */
export const monitoringChipLabels: Record<
  keyof typeof monitoringProtectionCopy,
  string
> = {
  idle: "Idle",
  monitoring: "Watching",
  alert: "Heads-up",
  cooldown: "Short pause",
};
