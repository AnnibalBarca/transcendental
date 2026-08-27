import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";

export default function PrivacyPolicyPage() {
	const { t } = useTranslation();
	const roles = t("legal.privacy.sections.contact.roles", { returnObjects: true }) as Record<string, string>;

	const emails: [keyof typeof roles, string][] = [
		["leadBackend", "madelvin@student.42lyon.fr"],
		["gameDesigner", "agantaum@student.42lyon.fr"],
		["cardArtist", "almeekel@student.42lyon.fr"],
		["frontendQA", "qutruch@student.42lyon.fr"],
		["devSecOps", "tarini@student.42lyon.fr"],
	];

	return (
		<main className="mx-auto max-w-3xl px-6 pb-12 pt-16 text-foreground">
			<h1 className="mb-2 text-3xl font-extrabold">{t("legal.privacy.title")}</h1>
			<p className="mb-10 text-sm text-muted-foreground">{t("legal.privacy.lastUpdated")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.data.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.data.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.gameData.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.gameData.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.messages.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.messages.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.retention.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.retention.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.sharing.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.sharing.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.rights.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.rights.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.privacy.sections.contact.title")}</h2>
			<p className="mb-4">{t("legal.privacy.sections.contact.intro")}</p>
			<ul className="mb-4 list-none">
				{emails.map(([roleKey, email]) => (
					<li key={roleKey}>
						{roles[roleKey]} : {email}
					</li>
				))}
			</ul>
			<p className="mb-8">
				{t("legal.privacy.sections.contact.outro")}{" "}
				<Link to="/terms" className="font-bold text-[#0B6E82] dark:text-[#3FCFE6] underline">
					{t("legal.terms.title").toLowerCase()}
				</Link>
				.
			</p>

			<Link to="/" className="font-bold text-[#0B6E82] dark:text-[#3FCFE6] underline">
				{t("legal.backHome")}
			</Link>
		</main>
	);
}