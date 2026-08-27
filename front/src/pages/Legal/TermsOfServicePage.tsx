import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";

export default function TermsOfServicePage() {
	const { t } = useTranslation();
	const conductItems = t("legal.terms.sections.conduct.items", { returnObjects: true }) as string[];

	return (
		<main className="mx-auto max-w-3xl px-6 pb-12 pt-16 text-foreground">
			<h1 className="mb-2 text-3xl font-extrabold">{t("legal.terms.title")}</h1>
			<p className="mb-10 text-sm text-muted-foreground">{t("legal.terms.lastUpdated")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.purpose.title")}</h2>
			<p className="mb-4">{t("legal.terms.sections.purpose.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.account.title")}</h2>
			<p className="mb-4">{t("legal.terms.sections.account.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.conduct.title")}</h2>
			<p className="mb-4">{t("legal.terms.sections.conduct.intro")}</p>
			<ul className="mb-4 list-disc pl-6">
				{conductItems.map((item, i) => (
					<li key={i} className="mb-1">{item}</li>
				))}
			</ul>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.currency.title")}</h2>
			<p className="mb-4">{t("legal.terms.sections.currency.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.suspension.title")}</h2>
			<p className="mb-4">{t("legal.terms.sections.suspension.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.availability.title")}</h2>
			<p className="mb-4">{t("legal.terms.sections.availability.body")}</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.privacy.title")}</h2>
			<p className="mb-4">
				{t("legal.terms.sections.privacy.body")}{" "}
				<Link to="/privacy" className="font-bold text-[#0B6E82] dark:text-[#3FCFE6] underline">
					{t("legal.privacy.title").toLowerCase()}
				</Link>
				.
			</p>

			<h2 className="mb-3 mt-8 text-xl font-bold">{t("legal.terms.sections.changes.title")}</h2>
			<p className="mb-8">{t("legal.terms.sections.changes.body")}</p>

			<Link to="/" className="font-bold text-[#0B6E82] dark:text-[#3FCFE6] underline">
				{t("legal.backHome")}
			</Link>
		</main>
	);
}