export interface SseEventBase {
	type: string;
}

export interface SseFriendRequest extends SseEventBase {
	type: "friend_request";
	from_user_id: string;
	username?: string;
}

export interface SseFriendAccepted extends SseEventBase {
	type: "friend_request_accepted";
	by_user_id: string;
	username?: string;
}

export interface SseFriendRefused extends SseEventBase {
	type: "friend_request_refused";
	by_user_id: string;
}

export interface SseFriendCancelled extends SseEventBase {
	type: "friend_request_cancelled";
	by_user_id: string;
}

export interface SseFriendRemoved extends SseEventBase {
	type: "friend_removed";
	by_user_id: string;
}

export interface SseNewMessage extends SseEventBase {
	type: "new_message";
	from_user_id: string;
	content: string;
	username?: string;
}

export interface SseProfilePictureUpdated extends SseEventBase {
	type: "profile_picture_updated";
	user_id: string;
	picture_id: string;
}

export type FriendSseEvent =
	| SseFriendRequest
	| SseFriendAccepted
	| SseFriendRefused
	| SseFriendCancelled
	| SseFriendRemoved
	| SseNewMessage
	| SseProfilePictureUpdated;

export type AnySseEvent = FriendSseEvent;
