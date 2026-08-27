import { useTranslation } from "react-i18next";
import GoogleLogin from "./GoogleLogin";
import FtLogin from "./FtLogin";

export default function SocialLogins() {
	const { t } = useTranslation();
	return (
		<>
			<div className="flex flex-col w-full gap-2.5 text-sm max-w-[min(80%,400px)]">
				<GoogleLogin />
				<FtLogin />
			</div>

			<div className="auth-divider max-w-[min(80%,400px)]">
				<span className="px-3 text-[#9ca3af] font-semibold text-sm uppercase select-none">{t("auth.or")}</span>
			</div>
		</>
	);
}
