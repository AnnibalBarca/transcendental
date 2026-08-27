import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { DoorOpen } from "lucide-react";
import { toast } from "@/components/ui/toast";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { roomService } from "@/features/room/services/roomService";
import { ThemeButton } from "./ThemeButton";

export function RoomWaitingPanel() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const { roomId } = useAuth();
	const [busy, setBusy] = useState(false);

	const joinLobby = () => {
		if (roomId) navigate(`/room/${roomId}`);
	};

	const handleLeave = async () => {
		setBusy(true);
		try {
			await roomService.leaveRoom();
			navigate("/play", { replace: true });
		} catch (e) {
			const msg = e instanceof Error ? e.message : t("room.leaveFailed");
			toast.add({
				title: t("room.leaveFailed"),
				description: msg,
				type: "error",
			});
		} finally {
			setBusy(false);
		}
	};

	return (
		<div className="flex w-full max-w-[680px] items-center justify-center gap-3 px-4">
			<ThemeButton
				type="button"
				onClick={joinLobby}
				texturePosition="center 98%"
				textureZoom={130}
				className="min-w-0 flex-1 px-4 py-3.5 text-sm tracking-[2px] uppercase"
			>
				{t("room.join")}
			</ThemeButton>

			<ThemeButton
				tone="red"
				type="button"
				onClick={handleLeave}
				disabled={busy}
				texturePosition="center 98%"
				textureZoom={130}
				className="min-w-0 flex-1 px-4 py-3.5 text-sm tracking-[2px] uppercase"
			>
				<DoorOpen className="size-4" />
				{t("room.leave")}
			</ThemeButton>
		</div>
	);
}