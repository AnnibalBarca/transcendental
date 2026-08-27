import type { Friend, FriendRequest } from "../services/friendService";
import type { AnySseEvent } from "../types/sse";

export interface ChatMessage {
	id: number;
	sender_id: string;
	content: string;
	created_at: string;
}

export interface FriendState {
	friends: Friend[];
	pendingRequests: FriendRequest[];
	sentRequests: FriendRequest[];
	blockedUsers: Friend[];
	messages: Record<string, ChatMessage[]>;
	unreadCounts: Record<string, number>;
	activeChatId: string | null;
	isLoading: boolean;
}

export type FriendAction =
	| { type: "set_loading"; payload: boolean }
		| {
				type: "load_all";
				payload: {
					friends: Friend[];
					pending: FriendRequest[];
					sent: FriendRequest[];
					blocked: Friend[];
					unreadCounts?: Record<string, number>;
				};
		  }
	| { type: "set_friends"; payload: Friend[] }
	| { type: "set_pending"; payload: FriendRequest[] }
	| { type: "set_sent"; payload: FriendRequest[] }
	| { type: "set_blocked"; payload: Friend[] }
	| { type: "add_pending"; payload: FriendRequest[] }
	| { type: "remove_friend"; payload: string }
	| { type: "add_friend"; payload: Friend }
	| { type: "accept_request"; payload: string }
	| { type: "refuse_request"; payload: string }
	| { type: "cancel_request"; payload: string }
	| { type: "block_user"; payload: string }
	| { type: "unblock_user"; payload: string }
	| { type: "add_message"; payload: { friendId: string; message: ChatMessage } }
	| {
			type: "set_messages";
			payload: { friendId: string; messages: ChatMessage[] };
	  }
	| { type: "clear_chat"; payload: string }
	| {
			type: "update_last_message";
			payload: { friendId: string; content: string; created_at: string };
	  }
	| {
			type: "set_unread_count";
			payload: { friendId: string; count: number };
	  }
	| { type: "increment_unread"; payload: string }
	| { type: "set_active_chat"; payload: string | null }
	| {
			type: "update_profile_picture";
			payload: { user_id: string; picture_id: string };
	  }
	| { type: "sse_event"; payload: AnySseEvent };

function removeFromAll(
	state: FriendState,
	userId: string,
): Partial<FriendState> {
	return {
		friends: state.friends.filter((f) => f.friend_id !== userId),
		pendingRequests: state.pendingRequests.filter((r) => r.user_id !== userId),
		sentRequests: state.sentRequests.filter((r) => r.user_id !== userId),
	};
}

export function friendReducer(
	state: FriendState,
	action: FriendAction,
): FriendState {
	switch (action.type) {
		case "set_loading":
			return { ...state, isLoading: action.payload };

		case "load_all": {
			const { friends, pending, sent, blocked, unreadCounts } = action.payload;
			const initialUnread: Record<string, number> = unreadCounts ?? {};
			for (const f of friends) {
				if (f.unread_count && !(f.friend_id in initialUnread)) {
					initialUnread[f.friend_id] = f.unread_count;
				}
			}
			return {
				...state,
				friends,
				pendingRequests: pending,
				sentRequests: sent,
				blockedUsers: blocked,
				unreadCounts: initialUnread,
				isLoading: false,
			};
		}

		case "set_friends":
			return { ...state, friends: action.payload };

		case "set_pending":
			return { ...state, pendingRequests: action.payload };

		case "set_sent":
			return { ...state, sentRequests: action.payload };

		case "set_blocked":
			return { ...state, blockedUsers: action.payload };

		case "add_pending":
			return { ...state, pendingRequests: action.payload };

		case "remove_friend": {
			const userId = action.payload;
			const nextMessages = { ...state.messages };
			delete nextMessages[userId];
			return {
				...state,
				...removeFromAll(state, userId),
				messages: nextMessages,
			};
		}

		case "add_friend": {
			const friend = action.payload;
			if (state.friends.some((f) => f.friend_id === friend.friend_id))
				return state;
			return {
				...state,
				friends: [...state.friends, friend],
				pendingRequests: state.pendingRequests.filter(
					(r) => r.user_id !== friend.friend_id,
				),
				sentRequests: state.sentRequests.filter(
					(r) => r.user_id !== friend.friend_id,
				),
			};
		}

		case "accept_request": {
			const userId = action.payload;
			return {
				...state,
				pendingRequests: state.pendingRequests.filter(
					(r) => r.user_id !== userId,
				),
			};
		}

		case "refuse_request":
			return {
				...state,
				pendingRequests: state.pendingRequests.filter(
					(r) => r.user_id !== action.payload,
				),
			};

		case "cancel_request":
			return {
				...state,
				sentRequests: state.sentRequests.filter(
					(r) => r.user_id !== action.payload,
				),
			};

		case "block_user": {
			const userId = action.payload;
			const nextMessages = { ...state.messages };
			delete nextMessages[userId];
			return {
				...state,
				...removeFromAll(state, userId),
				messages: nextMessages,
			};
		}

		case "unblock_user":
			return {
				...state,
				blockedUsers: state.blockedUsers.filter(
					(u) => u.friend_id !== action.payload,
				),
			};

		case "add_message": {
			const { friendId, message } = action.payload;
			return {
				...state,
				messages: {
					...state.messages,
					[friendId]: [...(state.messages[friendId] ?? []), message],
				},
				friends: state.friends.map((f) =>
					f.friend_id === friendId
						? {
								...f,
								last_message: {
									id: String(message.id),
									sender_id: message.sender_id,
									receiver_id: friendId,
									content: message.content,
									created_at: message.created_at,
								},
							}
						: f,
				),
			};
		}

		case "set_messages": {
			const { friendId, messages } = action.payload;
			return {
				...state,
				messages: { ...state.messages, [friendId]: messages },
			};
		}

		case "clear_chat": {
			const next = { ...state.messages };
			delete next[action.payload];
			return { ...state, messages: next };
		}

		case "update_last_message": {
			const { friendId, content, created_at } = action.payload;
			return {
				...state,
				friends: state.friends.map((f) =>
					f.friend_id === friendId
						? {
								...f,
								last_message: {
									...(f.last_message ?? {
										id: "",
										sender_id: "",
										receiver_id: "",
									}),
									content,
									created_at,
								},
							}
						: f,
				),
			};
		}

		case "set_unread_count": {
			const { friendId, count } = action.payload;
			return {
				...state,
				unreadCounts: { ...state.unreadCounts, [friendId]: count },
			};
		}

		case "increment_unread": {
			const friendId = action.payload;
			if (state.activeChatId === friendId) return state;
			const current = state.unreadCounts[friendId] ?? 0;
			return {
				...state,
				unreadCounts: { ...state.unreadCounts, [friendId]: current + 1 },
			};
		}

		case "set_active_chat": {
			return { ...state, activeChatId: action.payload };
		}

		case "update_profile_picture": {
			const { user_id, picture_id } = action.payload;
			return {
				...state,
				friends: state.friends.map((f) =>
					f.friend_id === user_id ? { ...f, picture_id } : f,
				),
				pendingRequests: state.pendingRequests.map((r) =>
					r.user_id === user_id ? { ...r, picture_id } : r,
				),
				sentRequests: state.sentRequests.map((r) =>
					r.user_id === user_id ? { ...r, picture_id } : r,
				),
				blockedUsers: state.blockedUsers.map((u) =>
					u.friend_id === user_id ? { ...u, picture_id } : u,
				),
			};
		}

		case "sse_event": {
			const event = action.payload;
			switch (event.type) {
				case "friend_request":
					return state;

				case "friend_request_accepted": {
					const { by_user_id, username } = event;
					if (!by_user_id) return state;
					return friendReducer(state, {
						type: "add_friend",
						payload: {
							friend_id: by_user_id,
							username,
							created_at: new Date().toISOString(),
						},
					});
				}

				case "friend_request_refused": {
					const userId = event.by_user_id;
					if (!userId) return state;
					return {
						...state,
						sentRequests: state.sentRequests.filter(
							(r) => r.user_id !== userId,
						),
					};
				}

				case "friend_request_cancelled": {
					const userId = event.by_user_id;
					if (!userId) return state;
					return {
						...state,
						pendingRequests: state.pendingRequests.filter(
							(r) => r.user_id !== userId,
						),
					};
				}

				case "friend_removed":
					return event.by_user_id
						? friendReducer(state, {
								type: "remove_friend",
								payload: event.by_user_id,
							})
						: state;

				case "new_message": {
					const { from_user_id, content } = event;
					const created_at = new Date().toISOString();
					return friendReducer(
						friendReducer(
							friendReducer(state, {
								type: "add_message",
								payload: {
									friendId: from_user_id,
									message: {
										id: Date.now(),
										sender_id: from_user_id,
										content,
										created_at,
									},
								},
							}),
							{
								type: "update_last_message",
								payload: {
									friendId: from_user_id,
									content,
									created_at,
								},
							},
						),
						{
							type: "increment_unread",
							payload: from_user_id,
						},
					);
				}

				case "profile_picture_updated": {
					const { user_id, picture_id } = event;
					if (!user_id || !picture_id) return state;
					return friendReducer(state, {
						type: "update_profile_picture",
						payload: { user_id, picture_id },
					});
				}

				default:
					return state;
			}
		}

		default:
			return state;
	}
}

export const initialFriendState: FriendState = {
	friends: [],
	pendingRequests: [],
	sentRequests: [],
	blockedUsers: [],
	messages: {},
	unreadCounts: {},
	activeChatId: null,
	isLoading: true,
};
