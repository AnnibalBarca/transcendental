import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../hooks/useSettings";
import { toast } from "@/components/ui/toast";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import type { UserSettings } from "../services/settingsService";
import { SETTINGS_LABEL, SETTINGS_SECTION } from "./styles/settingsStyles";

function ThemeForm({ settings }: { settings: UserSettings }) {
	const { t } = useTranslation();
	const { update } = useSettings();
	const [theme, setTheme] = useState<"dark" | "light">(
		settings.theme === "light" ? "light" : "dark",
	);

	const handleToggle = async () => {
		const next = theme === "dark" ? "light" : "dark";
		setTheme(next);
		try {
			await update({ theme: next });
		} catch (err) {
			setTheme(theme);
			toast.add(
				{
					title: t("settings.themeSaveFailed"),
					type: "error",
				}
			)
		}
	};

	return (
		<div className={SETTINGS_SECTION}>
			<span className={SETTINGS_LABEL}>{t("settings.theme")}</span>
			<ThemeButton
				type="button"
				className="self-start px-5 py-2.5 text-sm uppercase tracking-[1px]"
				texturePosition="center 98%"
				textureZoom={130}
				onClick={handleToggle}
			>
				{theme === "dark" ? t("settings.switchToLight") : t("settings.switchToDark")}
			</ThemeButton>
		</div>
	);
}

export default function ThemeToggle() {
	const { settings, loading } = useSettings();

	if (loading) return null;

	return (
		<ThemeForm
			key={settings?.theme ?? "dark"}
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