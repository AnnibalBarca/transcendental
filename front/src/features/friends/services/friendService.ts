import { API_SOCIAL } from "@/api/api";

function extractErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message;
	return String(error);
}

export interface FriendRequest {
	id: string;
	user_id: string;
	username?: string;
	picture_id?: string;
	created_at: string;
}

export interface Friend {
	friend_id: string;
	username?: string;
	picture_id?: string;
	created_at: string;
	last_message?: {
		id: string;
		sender_id: string;
		receiver_id: string;
		content: string;
		created_at: string;
	};
	unread_count?: number;
}

async function fetchRaw(url: string, options?: RequestInit): Promise<Response> {
	return fetch(url, {
		...options,
		credentials: "include",
		headers: {
			"Content-Type": "application/json",
			...(options?.headers ?? {}),
		},
	});
}

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
	const res = await fetchRaw(url, options);
	if (!res.ok) {
		const text = await res.text().catch(() => "Unknown error");
		throw new Error(text || `HTTP ${res.status}`);
	}
	return res.json() as Promise<T>;
}

export const friendService = {
	async getPendingRequests(): Promise<FriendRequest[]> {
		try {
			const data = await fetchJson<{
				status: number;
				requests?: FriendRequest[];
			}>(`${API_SOCIAL}/friend-requests`);
			return data.requests ?? [];
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async getSentRequests(): Promise<FriendRequest[]> {
		try {
			const data = await fetchJson<{
				status: number;
				requests?: FriendRequest[];
			}>(`${API_SOCIAL}/friend-requests/sent`);

			return data.requests ?? [];
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async getFriends(): Promise<Friend[]> {
		try {
			const data = await fetchJson<{ status: number; friends?: Friend[] }>(
				`${API_SOCIAL}/friends`,
			);
			return data.friends ?? [];
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async sendRequest(friend_username: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friend-requests`, {
				method: "POST",
				body: JSON.stringify({ friend_username }),
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async acceptRequest(friendId: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friend-requests/${friendId}/accept`, {
				method: "PATCH",
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async refuseRequest(friendId: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friend-requests/${friendId}/refuse`, {
				method: "PATCH",
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async cancelRequest(friendId: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friend-requests/${friendId}`, {
				method: "DELETE",
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async removeFriend(friendId: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friends/${friendId}`, {
				method: "DELETE",
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async getBlockedUsers(): Promise<Friend[]> {
		try {
			const data = await fetchJson<{ status: number; blocked?: Friend[] }>(
				`${API_SOCIAL}/friends/blocked`,
			);
			return data.blocked ?? [];
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async blockUser(friendId: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friends/${friendId}/block`, {
				method: "POST",
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async unblockUser(friendId: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friends/${friendId}/block`, {
				method: "DELETE",
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async markMessagesAsRead(friendId: string): Promise<number> {
		try {
			const data = await fetchJson<{
				status: number;
				marked_as_read?: number;
			}>(`${API_SOCIAL}/friends/${friendId}/messages/read`, {
				method: "POST",
			});
			return data.marked_as_read ?? 0;
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async getMessages(
		friendId: string,
		limit?: number,
		offset?: number,
	): Promise<Array<Record<string, unknown>>> {
		try {
			const params = new URLSearchParams();
			if (limit !== undefined) params.append("limit", String(limit));
			if (offset !== undefined) params.append("offset", String(offset));
			const qs = params.toString();
			const data = await fetchJson<{
				status: number;
				messages?: Array<Record<string, unknown>>;
			}>(`${API_SOCIAL}/friends/${friendId}/messages${qs ? `?${qs}` : ""}`);
			return data.messages ?? [];
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},

	async sendMessage(friendId: string, content: string): Promise<void> {
		try {
			await fetchJson<void>(`${API_SOCIAL}/friends/${friendId}/messages`, {
				method: "POST",
				body: JSON.stringify({ content }),
			});
		} catch (error) {
			throw new Error(extractErrorMessage(error));
		}
	},
};
