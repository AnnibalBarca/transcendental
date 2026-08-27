import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { SETTINGS_LABEL, SETTINGS_SECTION } from "./styles/settingsStyles";

export default function LegalSection() {
	const { t } = useTranslation();

	return (
		<div className={SETTINGS_SECTION}>
			<span className={SETTINGS_LABEL}>{t("settings.legal")}</span>

			<div className="flex flex-col gap-2">
				<Link to="/privacy" className="text-sm underline hover:opacity-80">
					{t("legal.privacy.title")}
				</Link>
				<Link to="/terms" className="text-sm underline hover:opacity-80">
					{t("legal.terms.title")}
				</Link>
			</div>
		</div>
	);
}