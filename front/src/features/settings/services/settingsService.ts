import { API_USER } from "@/features/auth/services/authService";
import extractErrorMessage from "@/features/auth/services/authService";

export interface UserSettings {
	username?: string;
	email?: string;
	bio: string;
	github: string;
	discord: string;
	twitter: string;
	is_private: boolean;
	theme: "dark" | "light";
}

export interface SettingsUpdate {
	bio?: string;
	github?: string;
	discord?: string;
	twitter?: string;
	is_private?: boolean;
	theme?: "dark" | "light";
}

async function getSettings(): Promise<UserSettings> {
	const response = await fetch(`${API_USER}/settings`, {
		credentials: "include",
	});

	if (!response.ok) {
		throw new Error("Failed to fetch settings");
	}

	const data = await response.json();
	return data.settings;
}

async function updateSettings(payload: SettingsUpdate): Promise<void> {
	const response = await fetch(`${API_USER}/settings`, {
		method: "PATCH",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(payload),
		credentials: "include",
	});

	if (!response.ok) {
		const msg = await extractErrorMessage(response, "Failed to update settings");
		throw new Error(msg);
	}
}

export const settingsService = {
	getSettings,
	updateSettings,
};