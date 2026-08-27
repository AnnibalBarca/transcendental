import { useEffect, useState } from "react";
import type { LucideIcon } from "lucide-react";
import {
	Users,
	ShieldCheck,
	KeyRound,
	Gauge,
	ArrowLeft,
	LayoutDashboard,
} from "lucide-react";
import { Link } from "react-router-dom";
import { cn } from "@/lib/utils";
import type { AdminSection } from "../types";
import { UsersPanel } from "./UsersPanel";
import { RolesPanel } from "./RolesPanel";
import { PermissionsPanel } from "./PermissionsPanel";
import { RateLimitsPanel } from "./RateLimitsPanel";

interface NavItem {
	id: AdminSection;
	label: string;
	icon: LucideIcon;
	description: string;
}

const NAV_ITEMS: NavItem[] = [
	{
		id: "users",
		label: "Utilisateurs",
		icon: Users,
		description: "Liste & gestion des comptes",
	},
	{
		id: "roles",
		label: "Rôles",
		icon: ShieldCheck,
		description: "Créer des rôles & attribuer des perms",
	},
	{
		id: "permissions",
		label: "Permissions",
		icon: KeyRound,
		description: "Créer des perms & lier des routes API",
	},
	{
		id: "rate-limits",
		label: "Rate limits",
		icon: Gauge,
		description: "Requêtes par minute par route",
	},
];

const SECTIONS: Record<AdminSection, string> = {
	users: "Utilisateurs",
	roles: "Rôles",
	permissions: "Permissions",
	"rate-limits": "Rate limits",
};

export function AdminLayout() {
	const [section, setSection] = useState<AdminSection>("users");

	useEffect(() => {
		document.documentElement.classList.add("dark");
		return () => {
			document.documentElement.classList.remove("dark");
		};
	}, []);

	return (
		<div className="flex min-h-screen flex-col bg-muted/30 md:flex-row">
			<aside className="border-border bg-background flex shrink-0 flex-col border-b p-3 md:sticky md:top-0 md:h-screen md:w-64 md:border-r md:border-b-0">
				<div className="flex items-center gap-2.5 px-2 py-3">
					<div className="bg-primary text-primary-foreground flex size-9 items-center justify-center rounded-lg">
						<LayoutDashboard className="size-5" />
					</div>
					<div className="min-w-0">
						<p className="truncate text-sm font-semibold">Admin Panel</p>
						<p className="text-muted-foreground text-xs">Transcendence</p>
					</div>
				</div>

				<nav className="mt-2 flex gap-1 overflow-x-auto md:flex-col md:overflow-visible">
					{NAV_ITEMS.map((item) => {
						const Icon = item.icon;
						const isActive = section === item.id;
						return (
							<button
								key={item.id}
								type="button"
								onClick={() => setSection(item.id)}
								className={cn(
									"flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
									"min-w-max text-left md:min-w-0",
									isActive
										? "bg-primary/10 text-primary"
										: "text-muted-foreground hover:bg-muted hover:text-foreground",
								)}
							>
								<Icon className="size-4.5 shrink-0" />
								<span className="hidden md:inline">
									<span className="block leading-tight">{item.label}</span>
									<span
										className={cn(
											"block text-xs font-normal",
											isActive
												? "text-primary/70"
												: "text-muted-foreground/70",
										)}
									>
										{item.description}
									</span>
								</span>
							</button>
						);
					})}
				</nav>

				<div className="mt-auto hidden border-t p-3 md:block">
					<Link
						to="/play"
						className="text-muted-foreground hover:text-foreground flex items-center gap-2 rounded-lg px-2 py-2 text-sm font-medium transition-colors hover:bg-muted"
					>
						<ArrowLeft className="size-4" />
						Retour au jeu
					</Link>
				</div>
			</aside>

			<main className="min-w-0 flex-1">
				<header className="border-border bg-background/80 sticky top-0 z-10 border-b px-6 py-4 backdrop-blur">
					<h1 className="text-lg font-semibold">
						{SECTIONS[section]}
					</h1>
					<p className="text-muted-foreground text-sm">
						{NAV_ITEMS.find((item) => item.id === section)?.description}
					</p>
				</header>

				<div className="p-4 md:p-6">
					{section === "users" && <UsersPanel />}
					{section === "roles" && <RolesPanel />}
					{section === "permissions" && <PermissionsPanel />}
					{section === "rate-limits" && <RateLimitsPanel />}
				</div>
			</main>
		</div>
	);
}
