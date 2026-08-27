import { useCallback, useEffect, useState } from "react";
import { Loader2, Pencil, Plus, Route as RouteIcon, ShieldCheck, Trash2 } from "lucide-react";
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
import type { AdminPermission, AdminRoute } from "../types";

function PermissionFormDialog({
	open,
	onOpenChange,
	permission,
	onSaved,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	permission: AdminPermission | null;
	onSaved: () => void;
}) {
	const [name, setName] = useState(permission?.name ?? "");
	const [description, setDescription] = useState(permission?.description ?? "");
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
			if (permission) {
				await adminService.updatePermission(permission.id, {
					name: name.trim(),
					description,
				});
			} else {
				await adminService.createPermission({ name: name.trim(), description });
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
					<DialogTitle>
						{permission ? "Modifier la permission" : "Nouvelle permission"}
					</DialogTitle>
					<DialogDescription>
						{permission ? permission.name : "Créer une permission personnalisée"}
					</DialogDescription>
				</DialogHeader>

				<div className="grid gap-4">
					<div className="grid gap-2">
						<Label htmlFor="perm-name">Nom</Label>
						<Input
							id="perm-name"
							value={name}
							onChange={(e) => setName(e.target.value)}
							placeholder="ex: users.ban"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor="perm-desc">Description</Label>
						<Input
							id="perm-desc"
							value={description}
							onChange={(e) => setDescription(e.target.value)}
							placeholder="ex: Permet de bannir un utilisateur"
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
						{permission ? "Enregistrer" : "Créer"}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}

function ManageRoutesForm({
	permission,
	routes,
	onClose,
	onSaved,
}: {
	permission: AdminPermission;
	routes: AdminRoute[];
	onClose: () => void;
	onSaved: () => void;
}) {
	const initial = (() => {
		const byLabel = new Map(routes.map((r) => [`${r.method} ${r.path}`, r.id]));
		const sel = new Set<number>();
		permission.routes.forEach((label) => {
			const id = byLabel.get(label);
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
			for (const id of toAdd) await adminService.addPermissionRoute(permission.id, id);
			for (const id of toRemove) await adminService.removePermissionRoute(permission.id, id);
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
					<RouteIcon className="size-5" />
					Routes de « {permission.name} »
				</DialogTitle>
				<DialogDescription>
					Sélectionnez les routes API accessibles par cette permission.
				</DialogDescription>
			</DialogHeader>

			<div className="flex max-h-72 flex-col gap-1 overflow-y-auto">
				{routes.length === 0 ? (
					<p className="text-muted-foreground py-8 text-center text-sm">
						Aucune route définie.
					</p>
				) : (
					routes.map((route) => (
						<div
							key={route.id}
							className="flex items-center justify-between gap-3 rounded-md px-2 py-2 hover:bg-muted"
						>
							<div className="flex min-w-0 items-center gap-2">
								<Badge variant="outline">{route.method}</Badge>
								<p className="truncate font-mono text-xs">{route.path}</p>
							</div>
							<Switch
								checked={selected.has(route.id)}
								onCheckedChange={() => toggle(route.id)}
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

function ManageRoutesDialog({
	open,
	onOpenChange,
	permission,
	routes,
	onSaved,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	permission: AdminPermission | null;
	routes: AdminRoute[];
	onSaved: () => void;
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[85vh] overflow-y-auto">
				{permission && (
					<ManageRoutesForm
						key={permission.id}
						permission={permission}
						routes={routes}
						onClose={() => onOpenChange(false)}
						onSaved={onSaved}
					/>
				)}
			</DialogContent>
		</Dialog>
	);
}

export function PermissionsPanel() {
	const [permissions, setPermissions] = useState<AdminPermission[]>([]);
	const [routes, setRoutes] = useState<AdminRoute[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [editPermission, setEditPermission] = useState<AdminPermission | null>(null);
	const [createOpen, setCreateOpen] = useState(false);
	const [managePermission, setManagePermission] = useState<AdminPermission | null>(null);
	const [deletePermission, setDeletePermission] = useState<AdminPermission | null>(null);
	const [deleting, setDeleting] = useState(false);
	const [deleteError, setDeleteError] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const [permsData, routesData] = await Promise.all([
				adminService.listPermissions(),
				adminService.listRoutes(),
			]);
			setPermissions(permsData);
			setRoutes(routesData);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Impossible de charger les permissions");
			setPermissions([]);
			setRoutes([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		let cancelled = false;
		(async () => {
			try {
				const [permsData, routesData] = await Promise.all([
					adminService.listPermissions(),
					adminService.listRoutes(),
				]);
				if (cancelled) return;
				setPermissions(permsData);
				setRoutes(routesData);
				setError(null);
			} catch (e) {
				if (cancelled) return;
				setError(e instanceof Error ? e.message : "Impossible de charger les permissions");
				setPermissions([]);
				setRoutes([]);
			} finally {
				if (!cancelled) setLoading(false);
			}
		})();
		return () => {
			cancelled = true;
		};
	}, []);

	const handleDelete = async () => {
		if (!deletePermission) return;
		setDeleting(true);
		setDeleteError(null);
		try {
			await adminService.deletePermission(deletePermission.id);
			setDeletePermission(null);
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
					{permissions.length} permission{permissions.length > 1 ? "s" : ""} ·{" "}
					{routes.length} route{routes.length > 1 ? "s" : ""}
				</p>
				<Button size="sm" onClick={() => setCreateOpen(true)}>
					<Plus />
					Nouvelle permission
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
					<p className="text-sm">Chargement des permissions…</p>
				</div>
			) : (
				<div className="bg-card rounded-xl border shadow-sm">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead>Permission</TableHead>
								<TableHead>Routes</TableHead>
								<TableHead className="text-right">Actions</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{permissions.length === 0 ? (
								<TableRow>
									<TableCell colSpan={3} className="text-muted-foreground py-12 text-center">
										Aucune permission trouvée
									</TableCell>
								</TableRow>
							) : (
								permissions.map((permission) => (
									<TableRow key={permission.id}>
										<TableCell>
											<div className="flex items-center gap-2">
												<ShieldCheck className="text-muted-foreground size-4" />
												<div className="min-w-0">
													<p className="truncate font-medium">{permission.name}</p>
													<p className="text-muted-foreground truncate text-xs">
														{permission.description || "—"}
													</p>
												</div>
											</div>
										</TableCell>
										<TableCell>
											<Badge variant="outline">{permission.routes.length}</Badge>
										</TableCell>
										<TableCell>
											<div className="flex items-center justify-end gap-1">
												<Button
													variant="ghost"
													size="sm"
													onClick={() => setManagePermission(permission)}
												>
													<RouteIcon />
													Routes
												</Button>
												<Button
													variant="ghost"
													size="icon-sm"
													title="Modifier"
													onClick={() => setEditPermission(permission)}
												>
													<Pencil />
												</Button>
												<Button
													variant="ghost"
													size="icon-sm"
													title="Supprimer"
													className="text-destructive hover:bg-destructive/10 hover:text-destructive"
													onClick={() => setDeletePermission(permission)}
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

			<PermissionFormDialog
				open={createOpen}
				onOpenChange={setCreateOpen}
				permission={null}
				onSaved={load}
			/>
			<PermissionFormDialog
				open={editPermission !== null}
				onOpenChange={(open) => {
					if (!open) setEditPermission(null);
				}}
				permission={editPermission}
				onSaved={load}
			/>
			<ManageRoutesDialog
				open={managePermission !== null}
				onOpenChange={(open) => {
					if (!open) setManagePermission(null);
				}}
				permission={managePermission}
				routes={routes}
				onSaved={load}
			/>

			<Dialog
				open={deletePermission !== null}
				onOpenChange={(open) => {
					if (!open) setDeletePermission(null);
				}}
			>
				<DialogContent className="sm:max-w-md">
					<DialogHeader>
						<DialogTitle>Supprimer la permission</DialogTitle>
						<DialogDescription>
							Voulez-vous vraiment supprimer la permission{" "}
							<strong>{deletePermission?.name}</strong> ? Cette action est irréversible.
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
							onClick={() => setDeletePermission(null)}
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
