import { useFriendContext } from "@/features/friends/context/FriendContext";
import { useTranslation } from "react-i18next";
import ProfilePicture from "@/components/ui/ProfilePicture";

interface BlockedListProps {
	onBack: () => void;
}

export default function BlockedList({ onBack }: BlockedListProps) {
	const { t } = useTranslation();
	const { blockedUsers, unblockUser } = useFriendContext();

	return (
		<div className="flex min-h-0 w-full max-w-[480px] flex-1 flex-col self-center overflow-hidden">
			<div className="flex h-[52px] shrink-0 items-center gap-2 border-b border-white/5 px-4">
				<button
					className="cursor-pointer rounded-md border-none p-1.5 text-[18px] leading-none text-white/55 transition-colors hover:bg-white/[0.06] hover:text-white"
					onClick={onBack}
					aria-label={t("friends.back")}
				>
					←
				</button>
				<span className="flex-1 text-[15px] font-semibold text-white">
					{t("friends.blockedUsers", { count: blockedUsers.length })}
				</span>
			</div>
			<div className="flex flex-1 flex-col gap-3 overflow-y-auto p-4">
				{blockedUsers.length === 0 && (
					<p className="p-4 text-center text-[13px] text-white/30">
						{t("friends.noBlocked")}
					</p>
				)}
				{blockedUsers.map((u) => (
					<div key={u.friend_id} className="flex items-center gap-3 py-2.5">
						<ProfilePicture
							pictureId={u.picture_id}
							size={40}
							className="shrink-0"
						/>
						<div className="flex min-w-0 flex-1 flex-col gap-0.5">
							<span className="truncate text-[15px] font-semibold text-white">
								{u.username ?? u.friend_id}
							</span>
						</div>
						<button
							className="shrink-0 cursor-pointer rounded-lg border-none bg-[rgba(74,222,128,0.12)] px-3.5 py-1.5 text-xs font-medium text-[#4ade80] transition-colors duration-150 hover:bg-[rgba(74,222,128,0.22)]"
							onClick={() => unblockUser(u.friend_id)}
							aria-label={t("friends.unblock")}
						>
							{t("friends.unblock")}
						</button>
					</div>
				))}
			</div>
		</div>
	);
}