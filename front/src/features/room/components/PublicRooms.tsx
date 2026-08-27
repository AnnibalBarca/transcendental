import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { toast } from "@/components/ui/toast";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { roomService } from "@/features/room/services/roomService";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { translateServerError } from "@/utils/serverError";

export default function PublicRooms() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const { publicRooms } = useAuth();
	const [snapshot, setSnapshot] = useState<Array<{
		id: string;
		title: string | null;
		host_username: string;
		player_count: number;
		max_players: number;
	}> | null>(null);
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		let cancelled = false;
		roomService
			.listPublicRooms()
			.then((data) => {
				if (!cancelled) setSnapshot(data.rooms);
			})
			.catch(() => {});
		return () => {
			cancelled = true;
		};
	}, []);

	const rooms = (publicRooms && publicRooms.length > 0 ? publicRooms : snapshot ?? [])
		.filter((r) => r.max_players === 2);

	const joinRoom = async (room_id: string) => {
		setBusy(true);
		setError(null);
		try {
			await roomService.joinRoom({ room_id });
			navigate(`/room/${room_id}`);
		} catch (e) {
			const rawMsg = e instanceof Error ? e.message : t("room.joinFailed");
			const msg = translateServerError(rawMsg, t);
			toast.add(
				{
					title: t("room.joinFailed"),
					description: msg,
					type: "error",
				}
			)
		} finally {
			setBusy(false);
		}
	};

	return (
		<div className="flex w-full flex-col items-center gap-2.5">
			{error && (
				<p className="m-0 rounded-[2px] bg-[#fee2e2] px-2.5 py-2 text-[0.8rem] text-[#b91c1c]">
					{error}
				</p>
			)}

			{rooms.length > 0 && (
				<div className="w-[95%] max-w-[680px] px-4 box-border">
					<h3 className="mx-0 mt-1.5 text-[0.9rem] uppercase tracking-[0.5px] text-[#60a5fa]">
						{t("room.publicGames")}
					</h3>
					<ul className="m-0 flex list-none flex-col gap-2 p-0">
						{rooms
							.filter((r) => r.max_players === 2)
							.map((r) => (
								<li
									key={r.id}
									className="flex items-center justify-between gap-2.5 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-3 py-2.5"
								>
									<div className="flex min-w-0 flex-1 flex-col gap-0.5">
										<span className="truncate text-[0.9rem] font-semibold">
											{r.title ?? t("room.game")}
										</span>
										<span className="text-[0.75rem] text-[#52545a]">
											{r.host_username} · {r.player_count}/{r.max_players}
										</span>
									</div>
									<ThemeButton
										type="button"
										onClick={() => joinRoom(r.id)}
										disabled={busy}
										texturePosition="center 98%"
										textureZoom={130}
										className="px-3 py-2 text-[0.8rem] tracking-[1px] uppercase"
									>
										{t("room.join")}
									</ThemeButton>
								</li>
							))}
					</ul>
				</div>
			)}
		</div>
	);
}
