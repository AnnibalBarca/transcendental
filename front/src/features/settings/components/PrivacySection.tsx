import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../hooks/useSettings";
import { toast } from "@/components/ui/toast";
import type { UserSettings } from "../services/settingsService";
import { SETTINGS_LABEL, SETTINGS_ROW, SETTINGS_SECTION } from "./styles/settingsStyles";

function PrivacyForm({ settings }: { settings: UserSettings }) {
	const { t } = useTranslation();
	const { update } = useSettings();
	const [isPrivate, setIsPrivate] = useState(settings.is_private);
	const [message, setMessage] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);

	const handleToggle = async () => {
		const next = !isPrivate;
		setIsPrivate(next);
		setError(null);
		setMessage(null);
		try {
			await update({ is_private: next });
			const msg = next ? t("settings.profileNowPrivate") : t("settings.profileNowPublic");
			setMessage(msg);
			toast.add(
				{
					title: msg,
					type: "success",
				}
			)
		} catch (err) {
			setIsPrivate(!next);
			setError(err instanceof Error ? err.message : t("settings.privacyUpdateFailed"));
			toast.add(
				{
					title: t("settings.privacyUpdateFailed"),
					description: err instanceof Error ? err.message : undefined,
					type: "error",
				}
			)
		}
	};

	return (
		<div className={SETTINGS_SECTION}>
			<span className={SETTINGS_LABEL}>{t("settings.privacy")}</span>

			<div className={SETTINGS_ROW}>
				<label className="relative inline-block h-6 w-[46px]">
					<input
						type="checkbox"
						checked={isPrivate}
						onChange={handleToggle}
						className="peer sr-only"
					/>
					<span className="absolute inset-0 cursor-pointer rounded-[4px] border border-[#334155]/60 bg-[#0f172a]/70 transition-colors duration-[250ms] before:absolute before:top-[2px] before:left-[3px] before:h-[18px] before:w-[18px] before:rounded-[3px] before:bg-white before:shadow-[0_2px_6px_rgba(0,0,0,0.4)] before:transition-transform before:duration-[250ms] peer-checked:border-transparent peer-checked:bg-blue-900 peer-checked:before:translate-x-[21px] peer-focus-visible:outline-2 peer-focus-visible:outline-[#60a5fa] peer-focus-visible:outline-offset-3"></span>
				</label>

				<span>{t("settings.privateProfile")}</span>
			</div>

			{message && <p className="m-0 text-xs text-emerald-400">{message}</p>}
			{error && <p className="m-0 text-xs text-red-400">{error}</p>}
		</div>
	);
}

export default function PrivacySection() {
	const { settings, loading } = useSettings();

	if (loading) return null;

	return (
		<PrivacyForm
			key={settings?.is_private ? "private" : "public"}
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