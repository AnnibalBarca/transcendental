import {
	createContext,
	useContext,
	useEffect,
	useCallback,
	useRef,
	useReducer,
} from "react";

import {
	friendService,
	type Friend,
	type FriendRequest,
} from "../services/friendService";
import { useSharedSse } from "../hooks/useSseShared";
import { useAuth } from "@/features/auth/hooks/useAuth";
import {
	friendReducer,
	initialFriendState,
	type FriendAction,
	type ChatMessage,
} from "../reducer/friendReducer";
import type { AnySseEvent } from "../types/sse";

interface FriendContextType {
	friends: Friend[];
	pendingRequests: FriendRequest[];
	sentRequests: FriendRequest[];
	blockedUsers: Friend[];
	isLoading: boolean;
	sseConnected: boolean;
	sseError: string | null;
	loadFriends: () => Promise<void>;
	sendRequest: (username: string) => Promise<void>;
	acceptRequest: (friendId: string) => Promise<void>;
	refuseRequest: (friendId: string) => Promise<void>;
	cancelRequest: (friendId: string) => Promise<void>;
	removeFriend: (friendId: string) => Promise<void>;
	blockUser: (friendId: string) => Promise<void>;
	unblockUser: (friendId: string) => Promise<void>;
	loadBlockedUsers: () => Promise<void>;
		messages: Record<string, ChatMessage[]>;
		sendMessage: (friendId: string, content: string) => Promise<void>;
		loadMessages: (friendId: string) => Promise<void>;
		unreadCounts: Record<string, number>;
		markMessagesAsRead: (friendId: string) => Promise<void>;
		activeChatId: string | null;
		setActiveChatId: (friendId: string | null) => void;
	}

const FriendContext = createContext<FriendContextType | undefined>(undefined);

export const FriendProvider = ({ children }: { children: React.ReactNode }) => {
	const { isAuthenticated, user } = useAuth();
	const isAccountValidated = user?.account_validated ?? false;
	const userId = user?.id;
	const sseUrl = userId ? `/api/notifications/sse/${userId}` : null;
	const [state, dispatch] = useReducer(friendReducer, initialFriendState);
	const initialLoadDone = useRef(false);

	const loadFriends = useCallback(async () => {
		dispatch({ type: "set_loading", payload: true });
		try {
		const [friends, pending, sent, blocked] = await Promise.all([
			friendService.getFriends(),
			friendService.getPendingRequests(),
			friendService.getSentRequests(),
			friendService.getBlockedUsers(),
		]);
		dispatch({
			type: "load_all",
			payload: { friends, pending, sent, blocked },
		});
		} catch (err) {
			dispatch({ type: "set_loading", payload: false });
		}
	}, []);

	useEffect(() => {
		if (!isAuthenticated || !isAccountValidated) {
			initialLoadDone.current = false;
			return;
		}
		if (!initialLoadDone.current) {
			initialLoadDone.current = true;
			loadFriends();
		}
	}, [isAuthenticated, isAccountValidated, loadFriends]);

	const handleSseEvent = useCallback((event: AnySseEvent) => {
		if (event.type === "friend_request") {
			friendService.getPendingRequests().then((requests) => {
				dispatch({ type: "set_pending", payload: requests });
			});
			return;
		}
		dispatch({ type: "sse_event", payload: event });
	}, []);

	const { connected: sseConnected, error: sseError } = useSharedSse<AnySseEvent>(
		handleSseEvent,
		isAccountValidated && !!sseUrl,
		sseUrl,
	);

	const sendRequest = useCallback(async (username: string) => {
		await friendService.sendRequest(username);
		const sent = await friendService.getSentRequests();
		dispatch({ type: "set_sent", payload: sent });
	}, []);

	const acceptRequest = useCallback(async (friendId: string) => {
		await friendService.acceptRequest(friendId);
		const [friends, pending] = await Promise.all([
			friendService.getFriends(),
			friendService.getPendingRequests(),
		]);
		dispatch({ type: "set_friends", payload: friends });
		dispatch({ type: "set_pending", payload: pending });
	}, []);

	const refuseRequest = useCallback(async (friendId: string) => {
		await friendService.refuseRequest(friendId);
		dispatch({ type: "refuse_request", payload: friendId });
	}, []);

	const cancelRequest = useCallback(async (friendId: string) => {
		await friendService.cancelRequest(friendId);
		dispatch({ type: "cancel_request", payload: friendId });
	}, []);

	const removeFriend = useCallback(async (friendId: string) => {
		await friendService.removeFriend(friendId);
		dispatch({ type: "remove_friend", payload: friendId });
	}, []);

	const blockUser = useCallback(async (friendId: string) => {
		await friendService.blockUser(friendId);
		dispatch({ type: "block_user", payload: friendId });
		const blocked = await friendService.getBlockedUsers();
		dispatch({ type: "set_blocked", payload: blocked });
	}, []);

	const unblockUser = useCallback(async (friendId: string) => {
		await friendService.unblockUser(friendId);
		dispatch({ type: "unblock_user", payload: friendId });
	}, []);

	const loadBlockedUsers = useCallback(async () => {
		try {
			const blocked = await friendService.getBlockedUsers();
			dispatch({ type: "set_blocked", payload: blocked });
		} catch (err) {
		}
	}, []);

	const sendMessage = useCallback(async (friendId: string, content: string) => {
		await friendService.sendMessage(friendId, content);
		dispatch({
			type: "add_message",
			payload: {
				friendId,
				message: {
					id: Date.now(),
					sender_id: "me",
					content,
					created_at: new Date().toISOString(),
				},
			},
		});
	}, []);

	const loadMessages = useCallback(
		async (friendId: string) => {
			try {
				const rawMsgs = await friendService.getMessages(friendId, 50, 0);
				const msgs = rawMsgs
					.map((m: Record<string, unknown>) => ({
						...m,
						sender_id: m.sender_id === user?.id ? "me" : m.sender_id,
					})) as ChatMessage[];
				msgs.sort(
					(a, b) =>
						new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
				);
				dispatch({
					type: "set_messages",
					payload: { friendId, messages: msgs },
				});
				await friendService.markMessagesAsRead(friendId);
				dispatch({
					type: "set_unread_count",
					payload: { friendId, count: 0 },
				});
			} catch (err) {
			}
		},
		[user?.id],
	);

	const markMessagesAsRead = useCallback(async (friendId: string) => {
		try {
			await friendService.markMessagesAsRead(friendId);
			dispatch({
				type: "set_unread_count",
				payload: { friendId, count: 0 },
			});
		} catch (err) {
		}
	}, []);

	const setActiveChatId = useCallback((friendId: string | null) => {
		dispatch({ type: "set_active_chat", payload: friendId });
	}, []);

	return (
		<FriendContext.Provider
			value={{
				friends: state.friends,
				pendingRequests: state.pendingRequests,
				sentRequests: state.sentRequests,
				blockedUsers: state.blockedUsers,
				isLoading: state.isLoading,
				sseConnected,
				sseError,
				loadFriends,
				sendRequest,
				acceptRequest,
				refuseRequest,
				cancelRequest,
				removeFriend,
				blockUser,
				unblockUser,
				loadBlockedUsers,
				messages: state.messages,
				sendMessage,
				loadMessages,
				unreadCounts: state.unreadCounts,
				markMessagesAsRead,
				activeChatId: state.activeChatId,
				setActiveChatId,
			}}
		>
			{children}
		</FriendContext.Provider>
	);
};

export const useFriendContext = () => {
	const context = useContext(FriendContext);
	if (!context)
		throw new Error("useFriendContext must be used within a FriendProvider");
	return context;
};
