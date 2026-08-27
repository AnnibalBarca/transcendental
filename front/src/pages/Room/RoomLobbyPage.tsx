import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { toast } from "@/components/ui/toast";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { useFriendContext } from "@/features/friends/context/FriendContext";
import ProfilePicture from "@/components/ui/ProfilePicture";
import { roomService, type RoomState } from "@/features/room/services/roomService";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { HomeBackground } from "@/components/HomeBackground";
import { translateServerError } from "@/utils/serverError";

const INVITE_PREFIX = "room_invite:";

export default function RoomLobbyPage() {
	const { t } = useTranslation();
	const { id } = useParams<{ id: string }>();
	const navigate = useNavigate();
	const [searchParams] = useSearchParams();
	const { user, userState, roomId, roomState } = useAuth();
	const { friends, sendMessage } = useFriendContext();
	const [room, setRoom] = useState<RoomState | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [busy, setBusy] = useState(false);
	const [copied, setCopied] = useState(false);
	const [invited, setInvited] = useState<Record<string, boolean>>({});
	const prevStateRef = useRef<string | null | undefined>(null);
	const joinAttemptedRef = useRef(false);

	const codeParam = searchParams.get("code");
	const joinParam = searchParams.get("join");
	const myId = user?.id ?? null;

	useEffect(() => {
		if (!id) return;
		let cancelled = false;
		const load = async () => {
			try {
				const data = await roomService.getRoomInfo(id);
				if (!cancelled) {
					setRoom(data.room);
					setError(null);
				}
			} catch (e) {
				if (!cancelled) {
					const msg = e instanceof Error ? e.message : t("room.roomNotFound");
					setError(msg);
					toast.add(
						{
							title: t("room.roomNotFound"),
							description: msg,
							type: "error",
						}
					)
				}
			}
		};
		load();
		const timer = setInterval(load, 2000);
		return () => {
			cancelled = true;
			clearInterval(timer);
		};
	}, [id]);

	const effectiveRoom = roomState && roomState.room_id === id ? roomState : room;

	const isInRoom = (roomState && roomState.room_id === id) || roomId === id;

	const inviteLink = effectiveRoom?.join_code
		? `${window.location.origin}/room/${id}?code=${effectiveRoom.join_code}`
		: null;

	useEffect(() => {
		if (!id || joinAttemptedRef.current) return;
		if (!codeParam && !joinParam) return;
		if (isInRoom) return;
		joinAttemptedRef.current = true;
		setBusy(true);
		const join = codeParam
			? roomService.joinRoom({ join_code: codeParam })
			: roomService.joinRoom({ room_id: id });
		join
			.then((res) => {
				navigate(`/room/${res.room_id}`, { replace: true });
			})
			.catch((e) => {
				const msg = translateServerError(e instanceof Error ? e.message : t("room.joinFailed"), t);
				toast.add(
					{
						title: t("room.joinFailed"),
						description: msg,
						type: "error",
					}
				)
			})
			.finally(() => setBusy(false));
	}, [id, codeParam, joinParam, isInRoom, navigate, t]);

	useEffect(() => {
		if (userState === "playing" && roomId) {
			navigate("/game/chess");
		}
	}, [userState, roomId, navigate]);

	useEffect(() => {
		const prev = prevStateRef.current;
		prevStateRef.current = userState;
		if ((prev === "waiting" || prev === "playing") && userState === "none") {
			navigate("/play");
		}
	}, [userState, navigate]);

	const handleLeave = async () => {
		setBusy(true);
		try {
			await roomService.leaveRoom();
			navigate("/play");
		} catch (e) {
			const msg = e instanceof Error ? e.message : t("room.leaveFailed");
			setError(msg);
			toast.add(
				{
					title: t("room.leaveFailed"),
					description: msg,
					type: "error",
				}
			)
		} finally {
			setBusy(false);
		}
	};

	const handleStart = async () => {
		if (!id) return;
		setBusy(true);
		setError(null);
		try {
			await roomService.startRoom(id);
		} catch (e) {
			const msg = e instanceof Error ? e.message : t("room.startFailed");
			setError(msg);
			toast.add(
				{
					title: t("room.startFailed"),
					description: msg,
					type: "error",
				}
			)
		} finally {
			setBusy(false);
		}
	};

	const handleKick = async (targetId: string, ban: boolean) => {
		if (!id) return;
		try {
			await roomService.kickPlayer(id, targetId, ban);
		} catch (e) {
			const msg = e instanceof Error ? e.message : t("room.kickFailed");
			setError(msg);
			toast.add(
				{
					title: t("room.kickFailed"),
					description: msg,
					type: "error",
				}
			)
		}
	};

	const handleCopyCode = () => {
		if (!inviteLink) return;
		navigator.clipboard
			.writeText(inviteLink)
			.then(() => {
				setCopied(true);
				setTimeout(() => setCopied(false), 1500);
			})
			.catch(() => setCopied(false));
	};

	const handleInviteFriend = async (friendId: string) => {
		if (!id) return;
		const joinCode = effectiveRoom?.join_code ?? null;
		try {
			await sendMessage(
				friendId,
				`${INVITE_PREFIX}${id}:${joinCode ?? ""}`,
			);
			setInvited((prev) => ({ ...prev, [friendId]: true }));
		} catch (e) {
			const msg = e instanceof Error ? e.message : t("room.inviteFailed");
			setError(msg);
			toast.add(
				{
					title: t("room.inviteFailed"),
					description: msg,
					type: "error",
				}
			)
		}
	};

	if (!effectiveRoom) {
		return (
			<div className="relative flex min-h-screen justify-center overflow-hidden bg-black pt-6 pb-12 text-white">
				<HomeBackground />
				<div className="relative z-10 flex w-full max-w-[min(80%,400px)] flex-col items-center self-start">
					<p className="text-[#aaa]">
						{error ? `${t("common.error")}: ${error}` : t("room.loading")}
					</p>
					{error && (
						<ThemeButton
							type="button"
							texturePosition="center 98%"
							textureZoom={130}
							className="w-full p-3 text-base tracking-[1px] uppercase"
							onClick={() => navigate("/play")}
						>
							{t("games.backToHome")}
						</ThemeButton>
					)}
				</div>
			</div>
		);
	}

	const isHost = myId === effectiveRoom.host_id;
	const isFull = effectiveRoom.players.length >= effectiveRoom.max_players;
	const roomJoinable =
		isInRoom &&
		effectiveRoom.status === "waiting" &&
		!isFull;

	return (
		<div className="relative flex min-h-screen justify-center overflow-hidden bg-black pt-6 pb-12 text-white">
			<HomeBackground />
			<div className="relative z-10 flex w-full max-w-[min(80%,400px)] flex-col items-center self-start">
				<div className="w-full text-center">
					<h1 className="m-0 text-[1.6rem] font-bold uppercase tracking-[1px] text-white">
						{effectiveRoom.title ?? t("room.newRoom")}
					</h1>
					<p className="mt-2 text-[0.9rem] text-[#52545a]">
						{effectiveRoom.private ? t("room.privateCode") : t("room.public")}
					</p>
				</div>

				{inviteLink && (
					<div className="mt-5 flex w-full items-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-3.5 py-3">
						<span className="text-[0.85rem] text-[#aaa]">{t("room.invitation")}</span>
						<span
							className="min-w-0 flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-[0.85rem] font-semibold text-[#60a5fa]"
							title={inviteLink}
						>
							{inviteLink}
						</span>
						<button
							type="button"
							className="cursor-pointer rounded-md border border-[#334155]/60 bg-transparent px-3 py-1.5 text-[0.8rem] font-semibold text-[#60a5fa] hover:bg-[#0f172a]/70"
							onClick={handleCopyCode}
						>
							{copied ? t("room.copied") : t("room.copy")}
						</button>
					</div>
				)}

				{error && (
					<p className="mt-3 w-full rounded-lg bg-[#fee2e2] p-2.5 text-[#b91c1c]">
						{error}
					</p>
				)}

				<p className="mb-2.5 mt-4 text-[0.85rem] font-semibold uppercase tracking-[0.5px] text-[#60a5fa]">
					{isFull
						? t("room.full")
						: t("room.waitingPlayers", { count: effectiveRoom.max_players - effectiveRoom.players.length })}
				</p>

				<ul className="m-0 flex w-full list-none flex-col gap-2 p-0">
					{effectiveRoom.players.map((p) => {
						const isMe = p.user_id === myId;
						const isRoomHost = p.user_id === effectiveRoom.host_id;
						return (
							<li
								key={p.user_id}
								className="flex items-center justify-between gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-3.5 py-3"
							>
								<span className="flex min-w-0 flex-1 items-center gap-2 overflow-hidden text-ellipsis whitespace-nowrap font-semibold">
									{p.username}
									{isRoomHost && (
										<span className="rounded-full bg-[#c9a227] px-2 py-0.5 text-[0.7rem] font-bold uppercase text-[#14161a]">
											{t("room.host")}
										</span>
									)}
									{isMe && (
										<span className="rounded-full bg-[#2f6fed] px-2 py-0.5 text-[0.7rem] font-bold uppercase text-white">
											{t("room.you")}
										</span>
									)}
								</span>
								{isHost && !isRoomHost && (
									<div className="flex gap-1.5">
										<button
											type="button"
											className="cursor-pointer rounded-md border border-[#334155]/60 bg-transparent px-2.5 py-1 text-[0.75rem] font-semibold text-[#ffd76a] hover:bg-[#0f172a]/70"
											onClick={() => handleKick(p.user_id, false)}
										>
											{t("room.kick")}
										</button>
										<button
											type="button"
											className="cursor-pointer rounded-md border border-[#334155]/60 bg-transparent px-2.5 py-1 text-[0.75rem] font-semibold text-[#ff6b6b] hover:bg-[#0f172a]/70"
											onClick={() => handleKick(p.user_id, true)}
										>
											{t("room.ban")}
										</button>
									</div>
								)}
							</li>
						);
					})}
				</ul>

				{friends.length > 0 && (
					<div className="mt-5 flex w-full flex-col gap-2">
						<p className="mb-1 text-[0.85rem] font-semibold uppercase tracking-[0.5px] text-[#60a5fa]">
							{t("room.inviteFriends")}
						</p>
						{!roomJoinable && (
							<p className="mb-1 text-xs text-[#52545a]">
								{t("room.notJoinable")}
								{effectiveRoom.status !== "waiting"
									? t("room.inProgress")
									: isFull
										? ` (${t("room.full")})`
										: ""}
							</p>
						)}
						<ul className="m-0 flex max-h-56 w-full list-none flex-col gap-2 overflow-y-auto p-0">
							{friends.map((f) => {
								const alreadyInRoom = effectiveRoom.players.some(
									(p) => p.user_id === f.friend_id,
								);
								const wasInvited = invited[f.friend_id];
								const disabled = !roomJoinable || alreadyInRoom || wasInvited;
								const label = alreadyInRoom
									? t("room.inRoom")
									: wasInvited
										? t("room.invited")
										: !roomJoinable
											? t("room.unavailable")
											: t("room.inviteToJoin");
								return (
									<li
										key={f.friend_id}
										className="flex items-center justify-between gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-3.5 py-2.5"
									>
										<div className="flex min-w-0 flex-1 items-center gap-2.5 overflow-hidden">
											<ProfilePicture
												pictureId={f.picture_id}
												size={32}
												className="shrink-0"
											/>
											<span className="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap font-semibold">
												{f.username ?? f.friend_id}
											</span>
										</div>
										<button
											type="button"
											className="shrink-0 cursor-pointer rounded-md border border-[#334155]/60 bg-transparent px-3 py-1.5 text-[0.8rem] font-semibold text-[#60a5fa] transition-colors duration-150 hover:bg-[#60a5fa]/10 disabled:cursor-not-allowed disabled:text-[#52545a]"
											onClick={() => handleInviteFriend(f.friend_id)}
											disabled={disabled}
										>
											{label}
										</button>
									</li>
								);
							})}
						</ul>
					</div>
				)}

				<div className="mt-5 flex w-full gap-2.5">
					{isHost ? (						<ThemeButton
							type="button"
							onClick={handleStart}
							disabled={busy || !isFull}
							className="flex-1 p-3 text-base tracking-[1px] uppercase"
						>
							{isFull ? t("room.start") : t("room.waitingForPlayers")}
						</ThemeButton>
					) : (
						<ThemeButton
							tone="red"
							type="button"
							className="flex-1 p-3 text-base tracking-[1px] uppercase"
							onClick={handleLeave}
							disabled={busy}
						>
							{t("room.leave")}
						</ThemeButton>
					)}
					{isHost && (
						<ThemeButton
							tone="red"
							type="button"
							className="flex-1 p-3 text-base tracking-[1px] uppercase"
							onClick={handleLeave}
							disabled={busy}
						>
							{t("room.cancel")}
						</ThemeButton>
					)}
				</div>
			</div>
		</div>
	);
}
