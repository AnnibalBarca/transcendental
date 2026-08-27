import { API_USER } from "@/features/auth/services/authService";
import extractErrorMessage from "@/features/auth/services/authService";

export interface PublicProfile {
	id: string;
	username?: string;
	email?: string;
	account_validated: boolean;
	email_validated: boolean;
	auth_provider: string;
	wallet: number;
	ranked_elo: number;
	level: number;
	xp: number;
	xp_progress: number;
	picture_id?: string;
	bio?: string;
	github?: string;
	discord?: string;
	twitter?: string;
	is_private?: boolean;
	state?: string;
}

async function getPublicProfile(identifier: string): Promise<PublicProfile> {
	const response = await fetch(
		`${API_USER}/users/${encodeURIComponent(identifier)}`,
		{
			credentials: "include",
		},
	);

	if (!response.ok) {
		const msg = await extractErrorMessage(response, "Failed to fetch profile");
		throw new Error(msg);
	}

	const data = await response.json();
	return data.user;
}

export const profileService = {
	getPublicProfile,
};