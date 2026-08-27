import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { LayoutDashboard } from "lucide-react";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { SETTINGS_LABEL, SETTINGS_SECTION, SETTINGS_SECTION_FULL, SETTINGS_SUBLABEL } from "./styles/settingsStyles";

export default function AdminPanelSection() {
	const { t } = useTranslation();
	const { hasPanelAccess } = useAuth();
	const navigate = useNavigate();

	if (!hasPanelAccess) return null;

	return (
		<div className={`${SETTINGS_SECTION} ${SETTINGS_SECTION_FULL}`}>
			<span className={SETTINGS_LABEL}>{t("settings.administration")}</span>
			<p className={SETTINGS_SUBLABEL}>
				{t("settings.adminPanelSubtitle")}
			</p>
			<ThemeButton
				type="button"
				className="h-[50px] w-full p-[9px] uppercase"
				texturePosition="center 98%"
				textureZoom={130}
				onClick={() => navigate("/panel")}
			>
				<LayoutDashboard className="size-4" />
				{t("settings.adminPanel")}
			</ThemeButton>
		</div>
	);
}
