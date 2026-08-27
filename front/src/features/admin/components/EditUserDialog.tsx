import { useEffect, useState } from "react";
import { Loader2, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { adminService } from "../services/adminService";
import type { AdminRole, AdminUser, UpdateUserPayload } from "../types";

interface EditUserDialogProps {
	user: AdminUser | null;
	open: boolean;
	onOpenChange: (open: boolean) => void;
	onSaved: () => void;
}

function EditUserForm({
	user,
	roles,
	onClose,
	onSaved,
}: {
	user: AdminUser;
	roles: AdminRole[];
	onClose: () => void;
	onSaved: () => void;
}) {
	const [username, setUsername] = useState(user.username ?? "");
	const [email, setEmail] = useState(user.email);
	const [accountValidated, setAccountValidated] = useState(user.account_validated);
	const [emailValidated, setEmailValidated] = useState(user.email_validated);
	const [isBanned, setIsBanned] = useState(user.is_banned ?? false);
	const [wallet, setWallet] = useState(String(user.wallet ?? 0));
	const [rankedElo, setRankedElo] = useState(String(user.ranked_elo ?? 1500));
	const [xp, setXp] = useState(String(user.xp ?? 0));
	const initialRoles = (() => {
		const byName = new Map(roles.map((r) => [r.name, r.id]));
		const sel = new Set<number>();
		(user.roles ?? []).forEach((name) => {
			const id = byName.get(name);
			if (id !== undefined) sel.add(id);
		});
		return sel;
	})();
	const [selectedRoles, setSelectedRoles] = useState<Set<number>>(initialRoles);
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const toggleRole = (id: number) => {
		setSelectedRoles((prev) => {
			const next = new Set(prev);
			if (next.has(id)) next.delete(id);
			else next.add(id);
			return next;
		});
	};

	const handleSubmit = async () => {
		setSubmitting(true);
		setError(null);
		const payload: UpdateUserPayload = {
			username: username.trim() || undefined,
			email: email.trim() || undefined,
			account_validated: accountValidated,
			email_validated: emailValidated,
			is_banned: isBanned,
			wallet: Number(wallet) || 0,
			ranked_elo: Number(rankedElo) || 0,
			xp: Number(xp) || 0,
		};
		try {
			await adminService.updateUser(user.id, payload);
			const toAdd = [...selectedRoles].filter((id) => !initialRoles.has(id));
			const toRemove = [...initialRoles].filter((id) => !selectedRoles.has(id));
			for (const id of toAdd) await adminService.addUserRole(user.id, id);
			for (const id of toRemove) await adminService.removeUserRole(user.id, id);
			onClose();
			onSaved();
		} catch (e) {
			setError(e instanceof Error ? e.message : "Une erreur est survenue");
		} finally {
			setSubmitting(false);
		}
	};

	return (
		<>
			<DialogHeader>
				<DialogTitle>Modifier l'utilisateur</DialogTitle>
				<DialogDescription>{user.username || user.email}</DialogDescription>
			</DialogHeader>

			<div className="grid gap-4">
				<div className="grid gap-2">
					<Label htmlFor="edit-username">Pseudo</Label>
					<Input
						id="edit-username"
						value={username}
						onChange={(e) => setUsername(e.target.value)}
						placeholder="Pseudo"
					/>
				</div>

				<div className="grid gap-2">
					<Label htmlFor="edit-email">Email</Label>
					<Input
						id="edit-email"
						type="email"
						value={email}
						onChange={(e) => setEmail(e.target.value)}
						placeholder="email@exemple.com"
					/>
				</div>

				<Separator />

				<div className="flex items-center justify-between gap-4">
					<Label htmlFor="edit-account-validated">
						Compte validé
						<span className="text-muted-foreground block text-xs font-normal">
							Profil complété
						</span>
					</Label>
					<Switch
						id="edit-account-validated"
						checked={accountValidated}
						onCheckedChange={(c) => setAccountValidated(c)}
					/>
				</div>

				<div className="flex items-center justify-between gap-4">
					<Label htmlFor="edit-email-validated">
						Email validé
						<span className="text-muted-foreground block text-xs font-normal">
							Adresse email confirmée
						</span>
					</Label>
					<Switch
						id="edit-email-validated"
						checked={emailValidated}
						onCheckedChange={(c) => setEmailValidated(c)}
					/>
				</div>

				<div className="flex items-center justify-between gap-4">
					<Label htmlFor="edit-banned" className="text-destructive">
						Bannir le compte
						<span className="text-muted-foreground block text-xs font-normal">
							Bloque la connexion et la réinscription avec cet email
						</span>
					</Label>
					<Switch
						id="edit-banned"
						checked={isBanned}
						onCheckedChange={(c) => setIsBanned(c)}
					/>
				</div>

				<Separator />

				<div className="grid gap-2">
					<Label className="flex items-center gap-2">
						<ShieldCheck className="size-4" />
						Rôles
					</Label>
					{roles.length === 0 ? (
						<p className="text-muted-foreground text-sm">Aucun rôle défini.</p>
					) : (
						<div className="flex flex-col gap-1">
							{roles.map((role) => (
								<div
									key={role.id}
									className="flex items-center justify-between gap-3 rounded-md px-2 py-2 hover:bg-muted"
								>
									<div className="min-w-0">
										<p className="truncate text-sm font-medium">{role.name}</p>
										<p className="text-muted-foreground truncate text-xs">
											{role.description || "—"}
										</p>
									</div>
									<Switch
										checked={selectedRoles.has(role.id)}
										onCheckedChange={() => toggleRole(role.id)}
									/>
								</div>
							))}
						</div>
					)}
				</div>

				<Separator />

				<div className="grid gap-2">
					<Label htmlFor="edit-wallet">Wallet</Label>
					<Input
						id="edit-wallet"
						type="number"
						min="0"
						value={wallet}
						onChange={(e) => setWallet(e.target.value)}
					/>
				</div>

				<div className="grid gap-2">
					<Label htmlFor="edit-ranked-elo">ELO Classé</Label>
					<Input
						id="edit-ranked-elo"
						type="number"
						value={rankedElo}
						onChange={(e) => setRankedElo(e.target.value)}
					/>
				</div>

				<div className="grid gap-2">
					<Label htmlFor="edit-xp">
						XP
						<span className="text-muted-foreground text-xs font-normal">
							{" "}
							(niveau recalculé automatiquement)
						</span>
					</Label>
					<Input
						id="edit-xp"
						type="number"
						min="0"
						value={xp}
						onChange={(e) => setXp(e.target.value)}
					/>
				</div>

				{error && (
					<p className="text-destructive rounded-md bg-destructive/10 px-3 py-2 text-sm">
						{error}
					</p>
				)}
			</div>

			<DialogFooter>
				<Button variant="outline" onClick={onClose} disabled={submitting}>
					Annuler
				</Button>
				<Button onClick={handleSubmit} disabled={submitting}>
					{submitting && <Loader2 className="animate-spin" />}
					Enregistrer
				</Button>
			</DialogFooter>
		</>
	);
}

export function EditUserDialog({ user, open, onOpenChange, onSaved }: EditUserDialogProps) {
	const [roles, setRoles] = useState<AdminRole[]>([]);
	const [rolesLoaded, setRolesLoaded] = useState(false);

	useEffect(() => {
		if (!user) return;
		let cancelled = false;
		(async () => {
			try {
				const data = await adminService.listRoles();
				if (!cancelled) {
					setRoles(data);
					setRolesLoaded(true);
				}
			} catch {
				if (!cancelled) {
					setRoles([]);
					setRolesLoaded(true);
				}
			}
		})();
		return () => {
			cancelled = true;
		};
	}, [user, open]);

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[85vh] overflow-y-auto">
				{user && rolesLoaded && (
					<EditUserForm
						key={user.id}
						user={user}
						roles={roles}
						onClose={() => onOpenChange(false)}
						onSaved={onSaved}
					/>
				)}
			</DialogContent>
		</Dialog>
	);
}
