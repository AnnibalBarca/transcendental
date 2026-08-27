import { useCallback, useEffect, useState } from "react";
import { KeyRound, Loader2, Pencil, Plus, ShieldCheck, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
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
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { adminService } from "../services/adminService";
import type { AdminPermission, AdminRole } from "../types";

function RoleFormDialog({
	open,
	onOpenChange,
	role,
	onSaved,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	role: AdminRole | null;
	onSaved: () => void;
}) {
	const [name, setName] = useState(role?.name ?? "");
	const [description, setDescription] = useState(role?.description ?? "");
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const handleSubmit = async () => {
		if (!name.trim()) {
			setError("Le nom est requis");
			return;
		}
		setSubmitting(true);
		setError(null);
		try {
			if (role) {
				await adminService.updateRole(role.id, { name: name.trim(), description });
			} else {
				await adminService.createRole({ name: name.trim(), description });
			}
			onOpenChange(false);
			onSaved();
		} catch (e) {
			setError(e instanceof Error ? e.message : "Une erreur est survenue");
		} finally {
			setSubmitting(false);
		}
	};

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{role ? "Modifier le rôle" : "Nouveau rôle"}</DialogTitle>
					<DialogDescription>
						{role ? role.name : "Créer un rôle personnalisé"}
					</DialogDescription>
				</DialogHeader>

				<div className="grid gap-4">
					<div className="grid gap-2">
						<Label htmlFor="role-name">Nom</Label>
						<Input
							id="role-name"
							value={name}
							onChange={(e) => setName(e.target.value)}
							placeholder="ex: moderator"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor="role-desc">Description</Label>
						<Input
							id="role-desc"
							value={description}
							onChange={(e) => setDescription(e.target.value)}
							placeholder="ex: Modérateur du chat"
						/>
					</div>
					{error && (
						<p className="text-destructive rounded-md bg-destructive/10 px-3 py-2 text-sm">
							{error}
						</p>
					)}
				</div>

				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>
						Annuler
					</Button>
					<Button onClick={handleSubmit} disabled={submitting}>
						{submitting && <Loader2 className="animate-spin" />}
						{role ? "Enregistrer" : "Créer"}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}

function ManagePermissionsForm({
	role,
	permissions,
	onClose,
	onSaved,
}: {
	role: AdminRole;
	permissions: AdminPermission[];
	onClose: () => void;
	onSaved: () => void;
}) {
	const initial = (() => {
		const byName = new Map(permissions.map((p) => [p.name, p.id]));
		const sel = new Set<number>();
		role.permissions.forEach((name) => {
			const id = byName.get(name);
			if (id !== undefined) sel.add(id);
		});
		return sel;
	})();
	const [selected, setSelected] = useState<Set<number>>(initial);
	const [original] = useState<Set<number>>(initial);
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const toggle = (id: number) => {
		setSelected((prev) => {
			const next = new Set(prev);
			if (next.has(id)) next.delete(id);
			else next.add(id);
			return next;
		});
	};

	const handleSave = async () => {
		setSubmitting(true);
		setError(null);
		try {
			const toAdd = [...selected].filter((id) => !original.has(id));
			const toRemove = [...original].filter((id) => !selected.has(id));
			for (const id of toAdd) await adminService.addRolePermission(role.id, id);
			for (const id of toRemove) await adminService.removeRolePermission(role.id, id);
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
				<DialogTitle className="flex items-center gap-2">
					<KeyRound className="size-5" />
					Permissions de « {role.name} »
				</DialogTitle>
				<DialogDescription>
					Cochez les permissions accordées à ce rôle.
				</DialogDescription>
			</DialogHeader>

			<div className="flex max-h-72 flex-col gap-1 overflow-y-auto">
				{permissions.length === 0 ? (
					<p className="text-muted-foreground py-8 text-center text-sm">
						Aucune permission définie.
					</p>
				) : (
					permissions.map((permission) => (
						<div
							key={permission.id}
							className="flex items-center justify-between gap-3 rounded-md px-2 py-2 hover:bg-muted"
						>
							<div className="min-w-0">
								<p className="truncate text-sm font-medium">{permission.name}</p>
								<p className="text-muted-foreground truncate text-xs">
									{permission.description || "—"}
								</p>
							</div>
							<Switch
								checked={selected.has(permission.id)}
								onCheckedChange={() => toggle(permission.id)}
							/>
						</div>
					))
				)}
			</div>

			{error && (
				<p className="text-destructive rounded-md bg-destructive/10 px-3 py-2 text-sm">
					{error}
				</p>
			)}

			<DialogFooter>
				<Button variant="outline" onClick={onClose} disabled={submitting}>
					Annuler
				</Button>
				<Button onClick={handleSave} disabled={submitting}>
					{submitting && <Loader2 className="animate-spin" />}
					Enregistrer
				</Button>
			</DialogFooter>
		</>
	);
}

function ManagePermissionsDialog({
	open,
	onOpenChange,
	role,
	permissions,
	onSaved,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	role: AdminRole | null;
	permissions: AdminPermission[];
	onSaved: () => void;
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[85vh] overflow-y-auto">
				{role && (
					<ManagePermissionsForm
						key={role.id}
						role={role}
						permissions={permissions}
						onClose={() => onOpenChange(false)}
						onSaved={onSaved}
					/>
				)}
			</DialogContent>
		</Dialog>
	);
}

export function RolesPanel() {
	const [roles, setRoles] = useState<AdminRole[]>([]);
	const [permissions, setPermissions] = useState<AdminPermission[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [editRole, setEditRole] = useState<AdminRole | null>(null);
	const [createOpen, setCreateOpen] = useState(false);
	const [manageRole, setManageRole] = useState<AdminRole | null>(null);
	const [deleteRole, setDeleteRole] = useState<AdminRole | null>(null);
	const [deleting, setDeleting] = useState(false);
	const [deleteError, setDeleteError] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const [rolesData, permsData] = await Promise.all([
				adminService.listRoles(),
				adminService.listPermissions(),
			]);
			setRoles(rolesData);
			setPermissions(permsData);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Impossible de charger les rôles");
			setRoles([]);
			setPermissions([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		let cancelled = false;
		(async () => {
			try {
				const [rolesData, permsData] = await Promise.all([
					adminService.listRoles(),
					adminService.listPermissions(),
				]);
				if (cancelled) return;
				setRoles(rolesData);
				setPermissions(permsData);
				setError(null);
			} catch (e) {
				if (cancelled) return;
				setError(e instanceof Error ? e.message : "Impossible de charger les rôles");
				setRoles([]);
				setPermissions([]);
			} finally {
				if (!cancelled) setLoading(false);
			}
		})();
		return () => {
			cancelled = true;
		};
	}, []);

	const handleDelete = async () => {
		if (!deleteRole) return;
		setDeleting(true);
		setDeleteError(null);
		try {
			await adminService.deleteRole(deleteRole.id);
			setDeleteRole(null);
			load();
		} catch (e) {
			setDeleteError(e instanceof Error ? e.message : "Une erreur est survenue");
		} finally {
			setDeleting(false);
		}
	};

	return (
		<div className="flex flex-col gap-4">
			<div className="flex items-center justify-between gap-2">
				<p className="text-muted-foreground text-sm">
					{roles.length} rôle{roles.length > 1 ? "s" : ""} · {permissions.length}{" "}
					permission{permissions.length > 1 ? "s" : ""}
				</p>
				<Button size="sm" onClick={() => setCreateOpen(true)}>
					<Plus />
					Nouveau rôle
				</Button>
			</div>

			{error && (
				<div className="flex flex-col items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 p-4">
					<p className="text-destructive text-sm">{error}</p>
					<Button size="sm" variant="outline" onClick={load}>
						Réessayer
					</Button>
				</div>
			)}

			{loading ? (
				<div className="flex flex-col items-center justify-center gap-2 py-16 text-muted-foreground">
					<Loader2 className="size-6 animate-spin" />
					<p className="text-sm">Chargement des rôles…</p>
				</div>
			) : (
				<div className="bg-card rounded-xl border shadow-sm">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead>Rôle</TableHead>
								<TableHead>Permissions</TableHead>
								<TableHead className="text-right">Actions</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{roles.length === 0 ? (
								<TableRow>
									<TableCell colSpan={3} className="text-muted-foreground py-12 text-center">
										Aucun rôle trouvé
									</TableCell>
								</TableRow>
							) : (
								roles.map((role) => (
									<TableRow key={role.id}>
										<TableCell>
											<div className="flex items-center gap-2">
												<ShieldCheck className="text-muted-foreground size-4" />
												<div className="min-w-0">
													<p className="truncate font-medium">{role.name}</p>
													<p className="text-muted-foreground truncate text-xs">
														{role.description || "—"}
													</p>
												</div>
											</div>
										</TableCell>
										<TableCell>
											<Badge variant="outline">{role.permissions.length}</Badge>
										</TableCell>
										<TableCell>
											<div className="flex items-center justify-end gap-1">
												<Button
													variant="ghost"
													size="sm"
													onClick={() => setManageRole(role)}
												>
													<KeyRound />
													Permissions
												</Button>
												<Button
													variant="ghost"
													size="icon-sm"
													title="Modifier"
													onClick={() => setEditRole(role)}
												>
													<Pencil />
												</Button>
												<Button
													variant="ghost"
													size="icon-sm"
													title="Supprimer"
													className="text-destructive hover:bg-destructive/10 hover:text-destructive"
													onClick={() => setDeleteRole(role)}
												>
													<Trash2 />
												</Button>
											</div>
										</TableCell>
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</div>
			)}

			<RoleFormDialog
				open={createOpen}
				onOpenChange={setCreateOpen}
				role={null}
				onSaved={load}
			/>
			<RoleFormDialog
				open={editRole !== null}
				onOpenChange={(open) => {
					if (!open) setEditRole(null);
				}}
				role={editRole}
				onSaved={load}
			/>
			<ManagePermissionsDialog
				open={manageRole !== null}
				onOpenChange={(open) => {
					if (!open) setManageRole(null);
				}}
				role={manageRole}
				permissions={permissions}
				onSaved={load}
			/>

			<Dialog
				open={deleteRole !== null}
				onOpenChange={(open) => {
					if (!open) setDeleteRole(null);
				}}
			>
				<DialogContent className="sm:max-w-md">
					<DialogHeader>
						<DialogTitle>Supprimer le rôle</DialogTitle>
						<DialogDescription>
							Voulez-vous vraiment supprimer le rôle{" "}
							<strong>{deleteRole?.name}</strong> ? Cette action est irréversible.
						</DialogDescription>
					</DialogHeader>

					{deleteError && (
						<p className="text-destructive rounded-md bg-destructive/10 px-3 py-2 text-sm">
							{deleteError}
						</p>
					)}

					<DialogFooter>
						<Button
							variant="outline"
							onClick={() => setDeleteRole(null)}
							disabled={deleting}
						>
							Annuler
						</Button>
						<Button variant="destructive" onClick={handleDelete} disabled={deleting}>
							{deleting && <Loader2 className="animate-spin" />}
							Supprimer
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	);
}
