import { useTranslation } from "react-i18next";
import { Shield, Link, User, LogOut } from "lucide-react";
import LanguageSelector from "@/components/LanguageSelector/LanguageSelector";
import ProfileSection from "./ProfileSection";
import SocialLinksSection from "./SocialLinksSection";
import PrivacySection from "./PrivacySection";
import LogoutButton from "./LogoutButton";
import DeleteAccountSection from "./DeleteAccountSection";
import ProviderSection from "./ProviderSection";
import LegalSection from "./LegalSection";
import AdminPanelSection from "./AdminPanelSection";
import {
	SETTINGS_GROUP,
	SETTINGS_GROUP_TITLE,
	SETTINGS_GRID,
	SETTINGS_HEADER,
	SETTINGS_ICON,
	SETTINGS_ICON_SVG,
	SETTINGS_PAGE,
	SETTINGS_TITLE,
} from "./styles/settingsStyles";

function SectionHeader({
	icon: Icon,
	title,
}: {
	icon: React.ComponentType<{ className?: string }>;
	title: string;
}) {
	return (
		<div className={SETTINGS_HEADER}>
			<div className={SETTINGS_ICON}>
				<Icon className={SETTINGS_ICON_SVG} />
			</div>
			<span className={SETTINGS_GROUP_TITLE}>{title}</span>
		</div>
	);
}

export default function Settings() {
	const { t } = useTranslation();

	return (
		<div className={SETTINGS_PAGE}>
			<div className="flex items-center justify-between gap-4">
				<h1 className={SETTINGS_TITLE}>{t("settings.title")}</h1>
				<LanguageSelector />
			</div>

			<div className={SETTINGS_GROUP}>
				<SectionHeader icon={User} title={t("settings.profile")} />
				<div className={SETTINGS_GRID}>
					<ProfileSection />
					<ProviderSection />
					<AdminPanelSection />
				</div>
			</div>

			<div className={SETTINGS_GROUP}>
				<SectionHeader icon={Link} title={t("settings.social")} />
				<div className={SETTINGS_GRID}>
					<SocialLinksSection />
				</div>
			</div>

			<div className={SETTINGS_GROUP}>
				<SectionHeader icon={Shield} title={t("settings.security")} />
				<div className={SETTINGS_GRID}>
					<PrivacySection />
					<LegalSection />
				</div>
			</div>

			<div className={SETTINGS_GROUP}>
				<SectionHeader icon={LogOut} title={t("settings.session")} />
				<div className={SETTINGS_GRID}>
					<LogoutButton />
					<DeleteAccountSection />
				</div>
			</div>
		</div>
	);
}