import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import i18n from "@/i18n";

export type MatchmakingState = "idle" | "loading" | "error";

export interface MatchmakingStatus {
	state: MatchmakingState;
	error: string | null;
}

const API_BASE = "/api/room";

async function apiPost(
	path: string,
	body?: unknown,
): Promise<{ ok: boolean; error?: string }> {
	try {
		const response = await fetch(`${API_BASE}${path}`, {
			method: "POST",
			credentials: "include",
			headers: {
				Accept: "application/json",
				...(body !== undefined
					? { "Content-Type": "application/json" }
					: {}),
			},
			body: body !== undefined ? JSON.stringify(body) : undefined,
		});

		const data = await response.json().catch(() => ({}));

		if (!response.ok) {
			return {
				ok: false,
				error: (data as { error?: string }).error || response.statusText,
			};
		}

		return { ok: true };
	} catch (err) {
		return {
			ok: false,
			error: err instanceof Error ? err.message : i18n.t("play.networkError"),
		};
	}
}

export function useMatchmaking() {
	const { t } = useTranslation();
	const [status, setStatus] = useState<MatchmakingStatus>({
		state: "idle",
		error: null,
	});

	const join = useCallback(async (timeControl: string) => {
		setStatus({ state: "loading", error: null });
		const result = await apiPost("/play_ranked", { time_control: timeControl });
		if (!result.ok) {
			setStatus({ state: "error", error: result.error || t("play.joinFailed") });
			return;
		}
		setStatus({ state: "idle", error: null });
	}, [t]);

	const cancel = useCallback(async () => {
		const result = await apiPost("/cancel_ranked");
		if (!result.ok) {
			setStatus((prev) => ({
				...prev,
				error: result.error || t("play.cancelFailed"),
			}));
			return;
		}
		setStatus({ state: "idle", error: null });
	}, [t]);

	const reset = useCallback(() => {
		setStatus({ state: "idle", error: null });
	}, []);

	return {
		status,
		join,
		cancel,
		reset,
	};
}
