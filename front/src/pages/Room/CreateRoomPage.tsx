import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { toast } from "@/components/ui/toast";
import { roomService } from "@/features/room/services/roomService";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { HomeBackground } from "@/components/HomeBackground";

type Visibility = "public" | "private";

const TIME_CONTROLS = [5, 10, 15];

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

export default function CreateRoomPage() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const [visibility, setVisibility] = useState<Visibility>("public");
	const [timeControl, setTimeControl] = useState(10);
	const [title, setTitle] = useState("");
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const handleCreate = async () => {
		setBusy(true);
		setError(null);
		try {
			const room = await roomService.createRoom({
				title: title.trim() || undefined,
				private: visibility === "private",
				max_players: 2,
				time_control: timeControl,
			});
			navigate(`/room/${room.room_id}`);
		} catch (e) {
			const msg = e instanceof Error ? e.message : t("room.createFailed");
			setError(msg);
			toast.add(
				{
					title: t("room.createFailed"),
					description: msg,
					type: "error",
				}
			)
		} finally {
			setBusy(false);
		}
	};

	return (
		<div className="relative flex min-h-screen flex-col items-center overflow-hidden bg-black pt-6 pb-12 text-white">
			<HomeBackground />
			<button
				type="button"
				className="absolute top-4 left-4 z-10 cursor-pointer rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-6 py-2.5 text-sm font-semibold uppercase tracking-[1px] text-[#60a5fa] transition-colors duration-150 hover:bg-[#1e293b] hover:text-white"
				onClick={() => navigate(-1)}
			>
				← {t("room.back")}
			</button>

			<div className="relative z-10 flex w-full max-w-[min(80%,400px)] flex-col items-center">
				<h1 className="mt-4 text-center text-[1.8rem] font-bold uppercase tracking-[1px] text-white/80">
					{t("room.newRoom")}
				</h1>
				<p className="mb-6 mt-2 text-[0.9rem] text-[#52545a]">{t("room.hostGame")}</p>

				<div className="mb-5 w-full">
					<label className="mb-2 block text-[0.8rem] font-bold uppercase tracking-[0.5px] text-[#aaa]">
						{t("room.timeControl")}
					</label>
					<div className="flex w-full gap-1 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-1">
						{TIME_CONTROLS.map((tc) => (
							<button
								key={tc}
								type="button"
								className={`cursor-pointer flex-1 rounded-[2px] bg-transparent px-1.5 py-2.5 text-[0.9rem] font-semibold text-[#aaa] transition-colors duration-150 hover:text-white ${
									timeControl === tc
										? "bg-blue-900! text-white! mix-blend-screen hover:text-white!"
										: ""
								}`}
								style={
									timeControl === tc
										? {
												backgroundImage: `url("${IMG_BASE}/carte/battlefield_0.svg")`,
												backgroundSize: "200%",
												backgroundPosition: "center 20%",
												filter: "grayscale(100%) sepia(40%) hue-rotate(190deg) saturate(150%)",
											}
										: undefined
								}
								onClick={() => setTimeControl(tc)}
							>
								{tc}
							</button>
						))}
					</div>
				</div>

				<div className="mb-5 w-full">
					<label className="mb-2 block text-[0.8rem] font-bold uppercase tracking-[0.5px] text-[#aaa]">
						{t("room.visibility")}
					</label>
					<div className="flex w-full gap-1 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-1">
						<button
							type="button"
							className={`cursor-pointer flex-1 rounded-[2px] bg-transparent px-1.5 py-2.5 text-[0.9rem] font-semibold text-[#aaa] transition-colors duration-150 hover:text-white ${
								visibility === "public"
									? "bg-blue-900! text-white! mix-blend-screen hover:text-white!"
									: ""
							}`}
							style={
								visibility === "public"
									? {
											backgroundImage: `url("${IMG_BASE}/carte/battlefield_0.svg")`,
											backgroundSize: "200%",
											backgroundPosition: "center 20%",
											filter: "grayscale(100%) sepia(40%) hue-rotate(190deg) saturate(150%)",
										}
									: undefined
							}
							onClick={() => setVisibility("public")}
						>
							{t("room.public")}
						</button>
						<button
							type="button"
							className={`cursor-pointer flex-1 rounded-[2px] bg-transparent px-1.5 py-2.5 text-[0.9rem] font-semibold text-[#aaa] transition-colors duration-150 hover:text-white ${
								visibility === "private"
									? "bg-blue-900! text-white! mix-blend-screen hover:text-white!"
									: ""
							}`}
							style={
								visibility === "private"
									? {
											backgroundImage: `url("${IMG_BASE}/carte/battlefield_0.svg")`,
											backgroundSize: "200%",
											backgroundPosition: "center 20%",
											filter: "grayscale(100%) sepia(40%) hue-rotate(190deg) saturate(150%)",
										}
									: undefined
							}
							onClick={() => setVisibility("private")}
						>
							{t("room.privateCode")}
						</button>
					</div>
					<p className="mt-2 text-[0.8rem] text-[#52545a]">
						{visibility === "private"
							? t("room.privateHint")
							: t("room.publicHint")}
					</p>
				</div>

				<div className="mb-5 w-full">
					<label className="mb-2 block text-[0.8rem] font-bold uppercase tracking-[0.5px] text-[#aaa]">
						{t("room.titleOptional")}
					</label>
					<input
						className="w-full rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-2.5 text-white focus:border-2 focus:border-[#60a5fa] focus:outline-none"
						value={title}
						onChange={(e) => setTitle(e.target.value)}
						placeholder={t("room.roomPlaceholder")}
						maxLength={40}
					/>
				</div>

				{error && (
					<p className="mb-3 w-full rounded-lg bg-[#fee2e2] p-2.5 text-[#b91c1c]">
						{error}
					</p>
				)}

				<ThemeButton
					type="button"
					onClick={handleCreate}
					disabled={busy}
					className="w-full p-3 text-base tracking-[1px] uppercase"
				>
					{busy ? t("room.creating") : t("room.create")}
				</ThemeButton>
			</div>
		</div>
	);
}
