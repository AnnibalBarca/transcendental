type TranslateFn = (key: string, options?: Record<string, unknown>) => string;

const RULES: Array<{ match: RegExp; key: string }> = [
	{ match: /already in (a|this) room/i, key: "room.alreadyInRoom" },
];

export function translateServerError(
	message: string | undefined,
	t: TranslateFn,
): string {
	if (!message) return t("common.error");
	for (const rule of RULES) {
		if (rule.match.test(message)) {
			return t(rule.key);
		}
	}
	return message;
}
