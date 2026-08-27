import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { profileService, type PublicProfile } from "@/features/player/services/profileService";
import ProfilePicture from "@/components/ui/ProfilePicture";
import { AtSign, ArrowLeft, GitBranch, MessageCircle, Check } from "lucide-react";
import { getRank, rankImageUrl } from "@/features/play/ranks";
import { HomeBackground } from "@/components/HomeBackground";

function ProfileView({ id, myId }: { id: string; myId?: string }) {
	const [profile, setProfile] = useState<PublicProfile | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [discordCopied, setDiscordCopied] = useState(false);

	const handleCopyDiscord = () => {
	if (!profile?.discord) return;
	navigator.clipboard.writeText(profile.discord);
	setDiscordCopied(true);
	setTimeout(() => setDiscordCopied(false), 1500);
	};

	useEffect(() => {
		let cancelled = false;
		profileService
			.getPublicProfile(id)
			.then((data) => {
				if (!cancelled) setProfile(data);
			})
			.catch((err) => {
				if (!cancelled) setError(err instanceof Error ? err.message : "Failed to load profile");
			});
		return () => {
			cancelled = true;
		};
	}, [id]);

	const elo = profile?.ranked_elo ?? 0;
	const rank = getRank(elo);
	const isOwn = !!profile && profile.id === myId;

	return (
		<div className="w-full">
			{error && (
				<div className="mx-auto mt-8 w-[min(90%,480px)] rounded-lg border border-[#f87171]/30 bg-[#f87171]/10 px-4 py-3 text-center text-sm text-[#f87171]">
					{error}
				</div>
			)}

			{!profile && !error && (
				<div className="mt-20 text-center text-[#52545a]">Loading profile…</div>
			)}

			{profile && (
				<div className="mx-auto flex w-full max-w-[720px] flex-col gap-5 px-4">
					<div className="flex items-center gap-5 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-6 backdrop-blur">
						<ProfilePicture
							pictureId={profile.picture_id}
							size={96}
							className="shrink-0"
						/>
						<div className="flex min-w-0 flex-1 flex-col gap-2">
							<h1 className="m-0 text-2xl font-extrabold tracking-tight">
								{profile.username ?? "Player"}
								{isOwn && (
									<span className="ml-2 rounded-full bg-[#60a5fa]/15 px-2 py-0.5 text-xs font-bold text-[#60a5fa]">
										You
									</span>
								)}
							</h1>

							{profile.bio ? (
								<p className="m-0 text-sm text-white/60">{profile.bio}</p>
							) : (
								<p className="m-0 text-sm text-white/30 italic">No bio yet.</p>
							)}

							{(profile.github || profile.discord || profile.twitter) && (
								<div className="flex items-center gap-3 pt-1">
									{profile.github && (
										<a
											href={`https://github.com/${profile.github}`}
											target="_blank"
											rel="noreferrer"
											className="flex h-8 w-8 items-center justify-center rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 text-white/60 transition-colors hover:border-[#60a5fa] hover:text-[#60a5fa]"
											title={`github.com/${profile.github}`}
										>
											<GitBranch size={16} />
										</a>
									)}
									{profile.discord && (
										<div className="relative">
											<button
												type="button"
												onClick={handleCopyDiscord}
												className="flex h-8 w-8 items-center justify-center rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 text-white/60 transition-colors hover:border-[#60a5fa] hover:text-[#60a5fa] cursor-pointer"
												title={`Discord: ${profile.discord} (click to copy)`}
											>
												{discordCopied ? (
													<Check size={16} className="text-green-400" />
												) : (
													<MessageCircle size={16} />
												)}
											</button>
											{discordCopied && (
												<span className="absolute -top-8 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-[2px] border border-[#334155]/60 bg-[#0f172a] px-2 py-1 text-xs font-semibold text-green-400 shadow-lg">
													Copied!
												</span>
											)}
										</div>
									)}
									{profile.twitter && (
										<a
											href={`https://x.com/${profile.twitter}`}
											target="_blank"
											rel="noreferrer"
											className="flex h-8 w-8 items-center justify-center rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 text-white/60 transition-colors hover:border-[#60a5fa] hover:text-[#60a5fa]"
											title={`x.com/${profile.twitter}`}
										>
											<AtSign size={16} />
										</a>
									)}
								</div>
							)}
						</div>
					</div>

					<div className="flex items-center justify-center gap-6 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 p-6 backdrop-blur">
						<img
							src={rankImageUrl(rank.image)}
							alt={rank.name}
							className="h-[120px] w-[120px] object-contain"
						/>
						<div className="flex flex-col items-center gap-1">
							<span className="text-2xl font-extrabold text-[#fbbf24]">
								{elo} ELO
							</span>
							<span className="text-sm font-semibold uppercase tracking-wider text-white/60">
								{rank.name}
							</span>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}

export default function PlayerProfilePage() {
	const { id } = useParams<{ id: string }>();
	const navigate = useNavigate();
	const { user } = useAuth();

	return (
		<div className="relative flex min-h-screen flex-col items-center overflow-hidden bg-black pt-6 pb-12 text-white">
			<HomeBackground />
			<div className="pointer-events-none absolute inset-0 z-0 flex items-center justify-center">
				<img
					src={`${import.meta.env.VITE_IMAGE_MINIO || "/img"}/carte/bestification_0.svg`}
					alt=""
					className="h-[min(70vh,520px)] w-auto max-w-[90vw] object-contain opacity-40"
				/>
			</div>
			<button
				type="button"
				className="absolute top-4 left-4 z-10 flex h-11 cursor-pointer items-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/80 px-4 text-sm font-semibold text-white transition-colors duration-150 hover:border-[#60a5fa] hover:text-[#60a5fa]"
				onClick={() => navigate(-1)}
			>
				<ArrowLeft className="h-4 w-4" />
				Back
			</button>

			{id && (
				<ProfileView key={id} id={id} myId={user?.id} />
			)}
		</div>
	);
}