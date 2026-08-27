import { Link, useLocation } from "react-router-dom";

const ROUTES_WITHOUT_NAVBAR = [
	"/login",
	"/signup",
	"/finish",
	"/validate-email",
	"/forgot-password",
	"/reset-password",
	"/privacy",
	"/terms",
];

const FOOTER =
	"flex flex-wrap items-center justify-center gap-x-6 gap-y-1 border-t border-border bg-background px-4 py-3 text-[12px] text-muted-foreground";

const FOOTER_LINK = "hover:text-foreground hover:underline";

export default function Footer() {
	const { pathname } = useLocation();

	if (!ROUTES_WITHOUT_NAVBAR.includes(pathname)) return null;

	return (
		<footer className={FOOTER}>
			<Link to="/privacy" className={FOOTER_LINK}>
				Confidentialité
			</Link>
			<Link to="/terms" className={FOOTER_LINK}>
				Conditions d'utilisation
			</Link>
		</footer>
	);
}
