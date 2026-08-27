import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useFriendContext } from "@/features/friends/context/FriendContext";
import ProfilePicture from "@/components/ui/ProfilePicture";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { roomService } from "@/features/room/services/roomService";
import { toast } from "@/components/ui/toast";
import { SquareButton } from "../SquareButton";
import { UserRound, LogIn, ArrowUp } from "lucide-react";

interface FriendChatProps {
	activeChatId: string;
	onBack: () => void;
	menuOpen: boolean;
	setMenuOpen: (v: boolean) => void;
	onRemoveFriend: (id: string) => void;
	onBlockFriend: (id: string) => void;
}

const INVITE_PREFIX = "room_invite:";

function parseRoomInvite(content: string): {
	roomId: string;
	joinCode: string | null;
} | null {
	if (!content.startsWith(INVITE_PREFIX)) return null;
	const rest = content.slice(INVITE_PREFIX.length);
	const [roomId, joinCode] = rest.split(":");
	if (!roomId) return null;
	return { roomId, joinCode: joinCode || null };
}

function RoomInviteCard({
	roomId,
	joinCode,
	onJoin,
}: {
	roomId: string;
	joinCode: string | null;
	onJoin: (roomId: string, joinCode: string | null) => void;
}) {
	const { t } = useTranslation();
	const [joinable, setJoinable] = useState<boolean | null>(null);

	useEffect(() => {
		let cancelled = false;
		roomService
			.getRoomInfo(roomId)
			.then((data) => {
				if (!cancelled) setJoinable(data.room?.status === "waiting");
			})
			.catch(() => {
				if (!cancelled) setJoinable(false);
			});
		return () => {
			cancelled = true;
		};
	}, [roomId]);

	const disabled = joinable === false;

	return (
		<div className="rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-3 text-sm leading-[1.4] break-words">
			<div className="flex items-center gap-2 text-[#60a5fa]">
				<LogIn size={15} />
				<span className="font-semibold">{t("friends.roomInvitation")}</span>
			</div>
			<button
				type="button"
				disabled={disabled}
				className="mt-2.5 w-full cursor-pointer rounded-[2px] border border-[#334155]/60 bg-blue-900 px-3 py-2 text-xs font-bold uppercase tracking-wide text-white transition-colors duration-150 hover:bg-blue-800 disabled:cursor-not-allowed disabled:bg-[#334155]/40 disabled:text-white/50"
				onClick={() => onJoin(roomId, joinCode)}
			>
				{joinable === null
					? t("friends.loading")
					: disabled
						? t("friends.unavailable")
						: t("friends.join")}
			</button>
		</div>
	);
}

export default function FriendChat({
	activeChatId,
	onBack,
	menuOpen,
	setMenuOpen,
	onRemoveFriend,
	onBlockFriend,
}: FriendChatProps) {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const { user } = useAuth();
	const {
		friends,
		messages,
		sendMessage,
		loadMessages,
	} = useFriendContext();

	const [draft, setDraft] = useState("");
	const scrollRef = useRef<HTMLDivElement | null>(null);

	const activeFriend = useMemo(
		() => friends.find((f) => f.friend_id === activeChatId) ?? null,
		[activeChatId, friends],
	);

	const chatMessages = messages[activeChatId] ?? [];

	useEffect(() => {
		loadMessages(activeChatId);
	}, [activeChatId, loadMessages]);

	useEffect(() => {
		const el = scrollRef.current;
		if (el) el.scrollTop = el.scrollHeight;
	}, [chatMessages.length]);

	const handleSend = async () => {
		if (!draft.trim()) return;
		try {
			await sendMessage(activeChatId, draft.trim());
			setDraft("");
		} catch (err) {
			toast.add(
				{
					title: t("friends.sendFailed"),
					type: "error",
				}
			)
		}
	};

	const handleJoin = (roomIdToJoin: string, joinCode: string | null) => {
		const url = joinCode
			? `/room/${roomIdToJoin}?code=${encodeURIComponent(joinCode)}`
			: `/room/${roomIdToJoin}?join=1`;
		navigate(url);
	};

	if (!activeFriend) return null;

	return (
		<>
			<div className="relative flex h-[52px] shrink-0 items-center gap-2 border-b border-white/5 px-4">
				<button
					className="cursor-pointer rounded-md border-none p-1.5 text-[18px] leading-none text-white/55 transition-colors hover:bg-white/[0.06] hover:text-white"
					onClick={onBack}
					aria-label={t("friends.back")}
				>
					←
				</button>
				<ProfilePicture
					pictureId={activeFriend.picture_id}
					size={32}
					className="shrink-0"
				/>
				<span className="min-w-0 flex-1 truncate text-[15px] font-semibold text-white">
					{activeFriend.username || activeFriend.friend_id}
				</span>

				<button
					className="cursor-pointer rounded-md border-none p-1.5 text-[18px] leading-none text-white/55 transition-colors hover:bg-white/[0.06] hover:text-white"
					onClick={(e) => {
						e.stopPropagation();
						setMenuOpen(!menuOpen);
					}}
					aria-label={t("friends.menu")}
				>
					⋮
				</button>
				{menuOpen && (
					<div
						data-friend-menu
						className="absolute top-[46px] right-3 z-10 min-w-[180px] rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/90 p-1 shadow-[0_4px_16px_rgba(0,0,0,0.5)]"
					>
						<button
							className="flex w-full cursor-pointer items-center gap-2 rounded-md border-none bg-transparent p-2 text-left text-[13px] text-white transition-colors hover:bg-white/[0.06]"
							onClick={() => {
								setMenuOpen(false);
								navigate(
									`/profile/${encodeURIComponent(
										activeFriend.username || activeFriend.friend_id,
									)}`,
								);
							}}
						>
							<UserRound size={14} />
							t("friends.viewProfile")
						</button>
						<button
							className="block w-full cursor-pointer rounded-md border-none bg-transparent p-2 text-left text-[13px] text-[#f87171] transition-colors hover:bg-white/[0.06]"
							onClick={() => onRemoveFriend(activeFriend.friend_id)}
						>
							{t("friends.removeFriend")}
						</button>
						<button
							className="block w-full cursor-pointer rounded-md border-none bg-transparent p-2 text-left text-[13px] text-[#f87171] transition-colors hover:bg-white/[0.06]"
							onClick={() => onBlockFriend(activeFriend.friend_id)}
						>
{t("friends.block")}
						</button>
					</div>
				)}
			</div>

			<div className="flex min-h-0 flex-1 flex-col">
				<div
					ref={scrollRef}
					className="flex flex-1 flex-col gap-2 overflow-y-auto px-3.5 py-4"
				>
					{chatMessages.length === 0 && (
						<div className="flex flex-1 flex-col items-center justify-center gap-2 py-8 text-center text-sm text-white/40">
							<span>{t("friends.noMessages")}</span>
							<span className="text-xs text-white/25">
								{t("friends.sayHi", { name: activeFriend.username || activeFriend.friend_id })}
							</span>
						</div>
					)}
					{chatMessages.map((msg) => {
						const invite = parseRoomInvite(msg.content);
						if (invite) {
							const mine = msg.sender_id === "me" || msg.sender_id === user?.id;
							return (
								<div
									key={msg.id}
									className={`max-w-[78%] ${
										mine ? "self-end" : "self-start"
									}`}
								>
									<RoomInviteCard
										roomId={invite.roomId}
										joinCode={invite.joinCode}
										onJoin={handleJoin}
									/>
								</div>
							);
						}
						return (
							<div
								key={msg.id}
								className={`max-w-[78%] rounded-[6px] px-3.5 py-2 text-sm leading-[1.4] break-words ${
									msg.sender_id === "me" || msg.sender_id === user?.id
										? "self-end border-b-4 border-b-transparent bg-blue-900 text-white [border-bottom-right-radius:4px]"
										: "self-start border-b-4 border-b-transparent bg-[#0f172a]/80 text-white/90 [border-bottom-left-radius:4px]"
								}`}
							>
								{msg.content}
							</div>
						);
					})}
				</div>
				<div className="flex shrink-0 items-center gap-2 border-t border-white/5 px-3.5 py-2.5">
					<input
						className="h-10 flex-1 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-4 text-sm text-white outline-none transition-[border-color] duration-150 placeholder:text-white/30 focus:border-[#60a5fa]"
						placeholder={t("friends.messagePlaceholder")}
						value={draft}
						onChange={(e) => setDraft(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === "Enter") handleSend();
						}}
					/>
					<SquareButton
						className="h-10 w-10"
						onClick={handleSend}
						aria-label={t("friends.send")}
					>
						<ArrowUp className="h-5 w-5 [stroke-width:2.5px]" />
					</SquareButton>
				</div>
			</div>
		</>
	);
}