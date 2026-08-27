export const API_LOGIN = "/api/auth";
export const API_USER = "/api/user";

export interface User {
	id: string;
	username?: string;
	email: string;
	state?: string;
	room_id?: string;
	chess_game_id?: string;
	chess_ws_url?: string;
  email_validated: boolean;
	account_validated: boolean;
	access_token_expires_in?: number;
	auth_provider?: string;
	ranked_elo?: number;
	level?: number;
	xp?: number;
	xp_progress?: number;
	picture_id?: string;
	has_panel_access?: boolean;
}

export default async function extractErrorMessage(
	response: Response,
	defaultMsg: string,
): Promise<string> {
	try {
		const data = await response.json();
		if (
			data &&
			typeof data.error === "string" &&
			data.error.trim().length > 0
		) {
			return data.error;
		}
		if (
			data &&
			typeof data.message === "string" &&
			data.message.trim().length > 0
		) {
			return data.message;
		}
	} catch (e) {
		void e;
	}
	try {
		const text = await response.text();
		if (text && text.trim().length > 0) {
			return text;
		}
	} catch (e) {
		void e;
	}
	return defaultMsg;
}

export const authService = {
	async register(email: string, password: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/register`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ email, password }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Registration failed");
			throw new Error(msg);
		}
	},

	async login(email: string, password: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/login/email`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ email, password }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Login failed");
			throw new Error(msg);
		}
	},

	async refresh(): Promise<void> {
		const response = await fetch(`${API_LOGIN}/refresh`, {
			method: "GET",
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, "Failed to refresh token");
			throw new Error(msg);
		}
	},

	async finishAccount(username: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/finish_account`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ username }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(
				response,
				"Failed to finish account setup",
			);
			throw new Error(msg);
		}
	},

	async googleVerify(code: string): Promise<void> {
		const response = await fetch(`${API_LOGIN}/google/code`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ code }),
			credentials: "include",
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || "Google login failed");
		}
	},

	async getMe(): Promise<User> {
		const response = await fetch(`${API_USER}/me`, {
			credentials: "include",
		});

		if (!response.ok) {
			throw new Error("Failed to fetch user");
		}

		const data = await response.json();
		return data.user;
	},

	async logout(): Promise<void> {
		const response = await fetch(`${API_LOGIN}/logout`, {
			method: "POST",
			credentials: "include",
		});

		if (!response.ok) {
			throw new Error("Logout failed");
		}
	},

	getAccessToken(): string | null {
		try {
			return localStorage.getItem("accessToken");
		} catch {
			return null;
		}
	},
};
