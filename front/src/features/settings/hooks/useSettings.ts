import { useCallback, useEffect, useState } from "react";
import {
	settingsService,
	type SettingsUpdate,
	type UserSettings,
} from "../services/settingsService";

export function useSettings() {
	const [settings, setSettings] = useState<UserSettings | null>(null);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		let cancelled = false;
		settingsService
			.getSettings()
			.then((data) => {
				if (!cancelled) setSettings(data);
			})
			.catch((err) => {
				if (!cancelled) setError(err instanceof Error ? err.message : String(err));
			})
			.finally(() => {
				if (!cancelled) setLoading(false);
			});
		return () => {
			cancelled = true;
		};
	}, []);

	const update = useCallback(async (payload: SettingsUpdate) => {
		await settingsService.updateSettings(payload);
		setSettings((prev) => (prev ? { ...prev, ...payload } : prev));
	}, []);

	return { settings, loading, error, update };
}