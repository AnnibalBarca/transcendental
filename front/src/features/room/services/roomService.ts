const API_ROOM = "/api/room";

async function extractErrorMessage(
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

export interface CreateRoomInput {
	title?: string;
	private: boolean;
	max_players: number;
	time_control?: number;
}

export interface PublicRoomListItem {
	id: string;
	title: string | null;
	host_username: string;
	player_count: number;
	max_players: number;
	created_at: number;
	private: boolean;
	join_code: string | null;
	mode: string;
}

export interface RoomState {
	room_id: string;
	title: string | null;
	private: boolean;
	join_code: string | null;
	host_id: string;
	players: { user_id: string; username: string }[];
	max_players: number;
	status: "waiting" | "playing" | "finished";
	time_control: number | null;
	game_type: string;
}

async function post<T>(path: string, body?: unknown): Promise<T> {
	const response = await fetch(`${API_ROOM}/${path}`, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		credentials: "include",
		body: body === undefined ? undefined : JSON.stringify(body),
	});

	if (!response.ok) {
		const msg = await extractErrorMessage(response, `Failed: ${path}`);
		throw new Error(msg);
	}
	return (await response.json()) as T;
}

async function get<T>(path: string): Promise<T> {
	const response = await fetch(`${API_ROOM}/${path}`, {
		method: "GET",
		credentials: "include",
	});

	if (!response.ok) {
		const msg = await extractErrorMessage(response, `Failed: ${path}`);
		throw new Error(msg);
	}
	return (await response.json()) as T;
}

export const roomService = {
	async createRoom(input: CreateRoomInput) {
		return post<{
			status: number;
			room_id: string;
			join_code: string | null;
			private: boolean;
			player_count: number;
			max_players: number;
			time_control: number | null;
			game_type: string;
		}>("create_room", input);
	},

	async joinRoom(input: { room_id?: string; join_code?: string }) {
		return post<{
			status: number;
			room_id: string;
			player_count: number;
			max_players: number;
		}>("join_room", input);
	},

	async leaveRoom() {
		return post<{ status: number }>("leave_room");
	},

	async startRoom(room_id: string) {
		return post<{ status: number }>("start_room", { room_id });
	},

	async kickPlayer(room_id: string, user_id: string, ban = false) {
		return post<{ status: number }>("kick_room", {
			room_id,
			user_id,
			ban,
		});
	},

	async getRoomInfo(room_id: string) {
		return post<{ status: number; room: RoomState }>("room_info", {
			room_id,
		});
	},

	async listPublicRooms() {
		return get<{ status: number; rooms: PublicRoomListItem[]; count: number }>(
			"list_public",
		);
	},

	async leaveGame(): Promise<void> {
		await this.leaveRoom();
	},
};
