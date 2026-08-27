import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import { useFriendContext } from "@/features/friends/context/FriendContext";
import FriendChat from "./friends/FriendChat";
import BlockedList from "./friends/BlockedList";
import ProfilePicture from "@/components/ui/ProfilePicture";
import { toast } from "@/components/ui/toast";
import { SquareButton } from "./SquareButton";

type View = "friends" | "blocked";

export default function FriendPanel() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const {
		friends,
		pendingRequests,
		sentRequests,
		blockedUsers,
		isLoading,
		sseConnected,
		sseError,
		sendRequest: sendFriendRequest,
		acceptRequest,
		refuseRequest,
		cancelRequest,
		removeFriend,
		blockUser,
		loadBlockedUsers,
		unreadCounts,
		activeChatId,
		setActiveChatId,
	} = useFriendContext();

	const [view, setView] = useState<View>("friends");
	const [addInput, setAddInput] = useState("");
	const [menuOpen, setMenuOpen] = useState(false);

	useEffect(() => {
		if (activeChatId && !friends.some((f) => f.friend_id === activeChatId)) {
			setActiveChatId(null);
			// eslint-disable-next-line react-hooks/set-state-in-effect -- sync menu state when chat closes
			setMenuOpen(false);
		}
	}, [friends, activeChatId, setActiveChatId]);

	useEffect(() => {
		if (!menuOpen) return;
		const onClick = (e: MouseEvent) => {
			const target = e.target as HTMLElement;
			if (!target.closest("[data-friend-menu]")) {
				setMenuOpen(false);
			}
		};
		window.addEventListener("click", onClick);
		return () => window.removeEventListener("click", onClick);
	}, [menuOpen]);

	const handleSendRequest = async () => {
		if (!addInput.trim()) return;
		try {
			await sendFriendRequest(addInput.trim());
			setAddInput("");
			toast.add(
				{
					title: t("friends.requestSent"),
					type: "success",
				}
			)
		} catch {
			toast.add(
				{
					title: t("friends.requestFailed"),
					description: t("friends.requestFailedDesc"),
					type: "error",
				}
			)
		}
	};

	const handleRemove = async (friendId: string) => {
		try {
			await removeFriend(friendId);
			setActiveChatId(null);
			setMenuOpen(false);
		} catch (err) {
			toast.add(
				{
					title: t("friends.removeFailed"),
					type: "error",
				}
			)
		}
	};

	const handleBlock = async (friendId: string) => {
		try {
			await blockUser(friendId);
			setActiveChatId(null);
			setMenuOpen(false);
		} catch (err) {
			toast.add(
				{
					title: t("friends.blockFailed"),
					type: "error",
				}
			)
		}
	};

	if (view === "blocked") {
		return (
			<BlockedList
				onBack={() => {
					setView("friends");
					loadBlockedUsers();
				}}
			/>
		);
	}

	return (
		<div className="flex min-h-0 w-full max-w-[480px] flex-1 flex-col self-center overflow-hidden">
			{activeChatId ? (
				<FriendChat
					activeChatId={activeChatId}
					onBack={() => setActiveChatId(null)}
					menuOpen={menuOpen}
					setMenuOpen={setMenuOpen}
					onRemoveFriend={handleRemove}
					onBlockFriend={handleBlock}
				/>
			) : (
				<>
					{!sseConnected && sseError && !isLoading && (
						<div
							style={{
								position: "absolute",
								inset: 0,
								background: "rgba(0,0,0,0.9)",
								zIndex: 100,
								display: "flex",
								flexDirection: "column",
								alignItems: "center",
								justifyContent: "center",
								padding: 24,
								textAlign: "center",
							}}
						>
							<h2 style={{ color: "#f87171", marginBottom: 12, fontSize: 18 }}>
								{t("friends.realtimeLost")}
							</h2>
							<p style={{ color: "#ccc", fontSize: 14 }}>{sseError}</p>
						</div>
					)}

					<div className="flex flex-1 flex-col gap-3 overflow-y-auto p-4">
					<div className="flex items-center gap-2.5 border-b border-white/5 pb-2">
						<input
							className="h-10 flex-1 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-3.5 text-sm text-white outline-none transition-[border-color] duration-150 placeholder:text-white/30 focus:border-[#60a5fa]"
							placeholder={t("friends.addFriend")}
								value={addInput}
								onChange={(e) => setAddInput(e.target.value)}
								onKeyDown={(e) => {
									if (e.key === "Enter") handleSendRequest();
								}}
							/>
							<SquareButton
								className="h-9 w-9"
								onClick={handleSendRequest}
								aria-label={t("friends.sendRequest")}
							>
								<Plus className="h-5 w-5 [stroke-width:2.5px]" />
							</SquareButton>
						</div>
						{isLoading && (
							<p className="m-0 mt-1 flex items-center gap-2 text-[11px] font-semibold uppercase text-[#888]">
								{t("friends.loading")}
							</p>
						)}

						{pendingRequests.length > 0 && (
							<>
								<p className="m-0 mt-1 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-widest text-white/35">
									{t("friends.requests")}{" "}
									<span className="inline-flex h-[18px] min-w-[18px] items-center justify-center rounded-[99px] bg-[#f87171] px-1.5 text-[10px] font-bold text-white">
										{pendingRequests.length}
									</span>
								</p>
								{pendingRequests.map((r) => (
									<div key={r.id} className="flex items-center gap-3 py-2.5">
										<ProfilePicture
											pictureId={r.picture_id}
											size={40}
											className="shrink-0"
										/>
										<div className="flex min-w-0 flex-1 flex-col gap-0.5">
											<span className="truncate text-[15px] font-semibold text-white">
												{r.username ?? r.user_id}
											</span>
										</div>
										<div className="flex shrink-0 gap-1.5">
											<button
												className="h-8 w-8 cursor-pointer rounded-lg border-none bg-[rgba(74,222,128,0.12)] text-[15px] text-[#4ade80] transition-colors duration-150 hover:bg-[rgba(74,222,128,0.22)]"
												onClick={() => acceptRequest(r.user_id)}
												aria-label={t("friends.accept")}
											>
												v
											</button>
											<button
												className="h-8 w-8 cursor-pointer rounded-lg border-none bg-[rgba(248,113,113,0.12)] text-[15px] text-[#f87171] transition-colors duration-150 hover:bg-[rgba(248,113,113,0.22)]"
												onClick={() => refuseRequest(r.user_id)}
												aria-label={t("friends.refuse")}
											>
												x
											</button>
											<button
												className="h-8 w-8 cursor-pointer rounded-lg border-none bg-[rgba(239,68,68,0.12)] text-sm text-[#ef4444] transition-colors duration-150 hover:bg-[rgba(239,68,68,0.22)]"
												onClick={() => blockUser(r.user_id)}
												aria-label={t("friends.block")}
												title={t("friends.blockUser")}
											>
												B
											</button>
										</div>
									</div>
								))}
							</>
						)}

						{sentRequests.length > 0 && (
							<>
								<p className="m-0 mt-1 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-widest text-white/35">
									{t("friends.pending")}
								</p>
								{sentRequests.map((p) => (
									<div key={p.id} className="flex items-center gap-3 py-2.5">
										<ProfilePicture
											pictureId={p.picture_id}
											size={40}
											className="shrink-0"
										/>
										<div className="flex min-w-0 flex-1 flex-col gap-0.5">
											<span className="truncate text-[15px] font-semibold text-white">
												{p.username ?? p.user_id}
											</span>
											<span className="truncate text-[11px] text-white/40">
												{t("friends.requestSent")}
											</span>
										</div>
										<button
											className="h-[30px] w-[30px] shrink-0 cursor-pointer rounded-lg border-none bg-white/[0.06] text-sm text-white/40 transition-colors duration-150 hover:bg-[rgba(248,113,113,0.2)] hover:text-[#f87171]"
											onClick={() => cancelRequest(p.user_id)}
											aria-label={t("friends.cancelRequest")}
										>
											x
										</button>
									</div>
								))}
							</>
						)}

						<p className="m-0 mt-1 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-widest text-white/35">
							{t("friends.friendsCount", { count: friends.length })}
						</p>
						{friends.length === 0 && !isLoading && (
							<p className="p-4 text-center text-[13px] text-white/30">
								{t("friends.noFriends")}
							</p>
						)}
						{friends.map((f) => {
							const unread = unreadCounts[f.friend_id] ?? 0;
							const lastMsg = f.last_message?.content;
							const lastText = lastMsg || t("friends.noMessages");
							return (
								<div
									key={f.friend_id}
									className="-mx-2.5 flex cursor-pointer items-center gap-3 rounded-[2px] px-3 py-3.5 transition-colors duration-150 hover:bg-white/[0.05]"
									onClick={() => {
										setActiveChatId(f.friend_id);
									}}
									role="button"
									tabIndex={0}
									onKeyDown={(e) => {
										if (e.key === "Enter") setActiveChatId(f.friend_id);
									}}
								>
									<ProfilePicture
										pictureId={f.picture_id}
										size={40}
										className="shrink-0"
									/>
									<div className="flex min-w-0 flex-1 flex-col gap-0.5">
										<div className="flex min-w-0 items-center gap-2.5">
											<button
												type="button"
												className="shrink-0 max-w-[40%] cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15px] font-semibold text-white transition-colors duration-150 hover:text-[#06b6d4]"
												onClick={(e) => {
													e.stopPropagation();
													navigate(`/profile/${encodeURIComponent(f.username ?? f.friend_id)}`);
												}}
											>
												{f.username ?? f.friend_id}
											</button>
											{unread > 0 && (
												<span className="inline-flex h-[18px] min-w-[18px] shrink-0 items-center justify-center rounded-[99px] bg-[#f87171] px-1.5 text-[10px] font-bold text-white">
													{unread}
												</span>
											)}
											<span className="min-w-0 flex-1 truncate self-center border-l border-white/15 pl-3 text-sm font-medium leading-[1.3] text-white/70">
												{lastText}
											</span>
										</div>
									</div>
								</div>
							);
						})}

						<div
							className="mt-auto cursor-pointer rounded-lg border-t border-white/5 px-2.5 py-3 text-[13px] text-white/40 transition-colors duration-150 hover:bg-white/[0.04] hover:text-white"
							onClick={() => {
								loadBlockedUsers();
								setView("blocked");
							}}
						>
							{t("friends.blockedUsers", { count: blockedUsers.length })}
						</div>
					</div>
				</>
			)}
		</div>
	);
}
