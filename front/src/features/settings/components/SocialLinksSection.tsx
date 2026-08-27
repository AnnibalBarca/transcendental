import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../hooks/useSettings";
import { toast } from "@/components/ui/toast";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import type { UserSettings } from "../services/settingsService";
import {
	SETTINGS_INPUT,
	SETTINGS_LABEL,
	SETTINGS_SECTION,
	SETTINGS_SECTION_FULL,
} from "./styles/settingsStyles";

function SocialLinksForm({ settings }: { settings: UserSettings }) {
	const { t } = useTranslation();
	const { update } = useSettings();
	const [social, setSocial] = useState({
		github: settings.github ?? "",
		discord: settings.discord ?? "",
		twitter: settings.twitter ?? "",
	});
	const [message, setMessage] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	const handleSave = async () => {
		setError(null);
		setMessage(null);
		try {
			await update({
				github: social.github.trim(),
				discord: social.discord.trim(),
				twitter: social.twitter.trim(),
			});
			setMessage(t("settings.usernamesSaved"));
			toast.add(
				{
					title: t("settings.usernamesSaved"),
					type: "success",
				}
			)
		} catch (err) {
			setError(err instanceof Error ? err.message : t("settings.usernamesSaveFailed"));
			toast.add(
				{
					title: t("settings.usernamesSaveFailed"),
					description: err instanceof Error ? err.message : undefined,
					type: "error",
				}
			)
		}
	};

	return (
		<div className={`${SETTINGS_SECTION} ${SETTINGS_SECTION_FULL}`}>
			<p className="m-0 text-xs text-white/40">
				{t("settings.socialsHint")}
			</p>

			<label className={SETTINGS_LABEL} htmlFor="github">
				{t("settings.githubLabel")}
			</label>
			<input
				id="github"
				className={SETTINGS_INPUT}
				value={social.github}
				onChange={(e) => setSocial({ ...social, github: e.target.value })}
			/>

			<label className={SETTINGS_LABEL} htmlFor="discord">
				{t("settings.discordLabel")}
			</label>
			<input
				id="discord"
				className={SETTINGS_INPUT}
				value={social.discord}
				onChange={(e) => setSocial({ ...social, discord: e.target.value })}
			/>

			<label className={SETTINGS_LABEL} htmlFor="twitter">
				{t("settings.twitterLabel")}
			</label>
			<input
				id="twitter"
				className={SETTINGS_INPUT}
				value={social.twitter}
				onChange={(e) => setSocial({ ...social, twitter: e.target.value })}
			/>

			{message && <p className="m-0 text-xs text-emerald-400">{message}</p>}
			{error && <p className="m-0 text-xs text-red-400">{error}</p>}

			<ThemeButton
				type="button"
				className="self-start px-5 py-2.5 text-sm uppercase tracking-[1px]"
				texturePosition="center 98%"
				textureZoom={130}
				onClick={handleSave}
			>
				{t("settings.saveUsernames")}
			</ThemeButton>
		</div>
	);
}

export default function SocialLinksSection() {
	const { settings, loading } = useSettings();

	if (loading) return null;

	return (
		<SocialLinksForm
			key={settings?.github ?? "none"}
			settings={
				settings ?? {
					username: "",
					bio: "",
					github: "",
					discord: "",
					twitter: "",
					is_private: false,
					theme: "dark",
				}
			}
		/>
	);
}