import { useCallback, useEffect, useState } from "react";
import {
	ChevronLeft,
	ChevronRight,
	Loader2,
	Pencil,
	RefreshCw,
	Trash2,
	Users as UsersIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import ProfilePicture from "@/components/ui/ProfilePicture";
import { adminService } from "../services/adminService";
import type { AdminUser } from "../types";
import { EditUserDialog } from "./EditUserDialog";
import { DeleteUserDialog } from "./DeleteUserDialog";
import { CardsManagerDialog, GrantCardButton } from "./GrantCardDialog";

const PAGE_SIZE = 50;

function RoleBadge({ user }: { user: AdminUser }) {
	if (user.roles.length === 0) {
		return <Badge variant="secondary">aucun rôle</Badge>;
	}
	const isAdmin = user.roles.includes("admin");
	return (
		<div className="flex max-w-40 flex-wrap gap-1">
			{isAdmin && <Badge variant="default">admin</Badge>}
			{user.roles
				.filter((r) => r !== "admin")
				.map((role) => (
					<Badge key={role} variant="outline">
						{role}
					</Badge>
				))}
		</div>
	);
}

function StatusBadges({ user }: { user: AdminUser }) {
	return (
		<div className="flex gap-1">
			{user.is_banned && <Badge variant="destructive">Banni</Badge>}
			<Badge variant={user.account_validated ? "success" : "warning"}>
				{user.account_validated ? "Validé" : "À compléter"}
			</Badge>
			{user.email_validated && <Badge variant="outline">Email</Badge>}
		</div>
	);
}

export function UsersPanel() {
	const [users, setUsers] = useState<AdminUser[]>([]);
	const [total, setTotal] = useState(0);
	const [page, setPage] = useState(0);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const [editUser, setEditUser] = useState<AdminUser | null>(null);
	const [deleteUser, setDeleteUser] = useState<AdminUser | null>(null);
	const [grantCardUser, setGrantCardUser] = useState<AdminUser | null>(null);

	const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

	const loadUsers = useCallback(async (pageIndex: number) => {
		try {
			const data = await adminService.listUsers(pageIndex * PAGE_SIZE, PAGE_SIZE);
			setUsers(data.users);
			setTotal(data.total);
			setError(null);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Impossible de charger les utilisateurs");
			setUsers([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		let cancelled = false;
		(async () => {
			try {
				const data = await adminService.listUsers(page * PAGE_SIZE, PAGE_SIZE);
				if (cancelled) return;
				setUsers(data.users);
				setTotal(data.total);
				setError(null);
				if (page >= Math.max(1, Math.ceil(data.total / PAGE_SIZE))) {
					setPage(0);
				}
			} catch (e) {
				if (cancelled) return;
				setError(e instanceof Error ? e.message : "Impossible de charger les utilisateurs");
				setUsers([]);
			} finally {
				if (!cancelled) setLoading(false);
			}
		})();
		return () => {
			cancelled = true;
		};
	}, [page]);

	const refresh = () => {
		loadUsers(page);
	};

	const gotoPage = (index: number) => {
		const next = Math.min(Math.max(0, index), totalPages - 1);
		if (next === page) return;
		setLoading(true);
		setPage(next);
	};

	const firstVisible = total === 0 ? 0 : page * PAGE_SIZE + 1;
	const lastVisible = Math.min((page + 1) * PAGE_SIZE, total);

	return (
		<div className="flex flex-col gap-4">
			<Card>
				<CardHeader>
					<CardTitle className="flex items-center gap-2">
						<UsersIcon className="text-muted-foreground size-5" />
						Liste des utilisateurs
					</CardTitle>
					<CardDescription>{total} compte{total > 1 ? "s" : ""}</CardDescription>
				</CardHeader>
				<CardContent>
					<div className="mb-4 flex items-center justify-between gap-2">
						<p className="text-muted-foreground text-sm">
							{firstVisible}–{lastVisible} / {total}
						</p>
						<Button
							variant="outline"
							size="sm"
							onClick={refresh}
							disabled={loading}
						>
							<RefreshCw className={loading ? "animate-spin" : ""} />
							Rafraîchir
						</Button>
					</div>

					{error && (
						<div className="flex flex-col items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 p-4">
							<p className="text-destructive text-sm">{error}</p>
							<Button size="sm" variant="outline" onClick={refresh}>
								Réessayer
							</Button>
						</div>
					)}

					{loading ? (
						<div className="flex flex-col items-center justify-center gap-2 py-16 text-muted-foreground">
							<Loader2 className="size-6 animate-spin" />
							<p className="text-sm">Chargement des utilisateurs…</p>
						</div>
					) : users.length === 0 ? (
						<div className="flex flex-col items-center justify-center gap-2 py-16 text-muted-foreground">
							<UsersIcon className="size-8 opacity-40" />
							<p className="text-sm">Aucun utilisateur trouvé</p>
						</div>
					) : (
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Utilisateur</TableHead>
									<TableHead>Rôle</TableHead>
									<TableHead>Statut</TableHead>
									<TableHead>Niveau</TableHead>
									<TableHead>ELO</TableHead>
									<TableHead className="text-right">Wallet</TableHead>
									<TableHead className="text-right">Actions</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{users.map((user) => (
									<TableRow key={user.id}>
										<TableCell>
											<div className="flex items-center gap-3">
												<ProfilePicture pictureId={user.picture_id} size={32} />
												<div className="min-w-0">
													<p className="truncate font-medium">
														{user.username || "—"}
													</p>
													<p className="text-muted-foreground truncate text-xs">
														{user.email}
													</p>
												</div>
											</div>
										</TableCell>
										<TableCell>
											<RoleBadge user={user} />
										</TableCell>
										<TableCell>
											<StatusBadges user={user} />
										</TableCell>
										<TableCell>{user.level}</TableCell>
										<TableCell>{user.ranked_elo}</TableCell>
										<TableCell className="text-right">
											{user.wallet.toLocaleString("fr-FR")}
										</TableCell>
										<TableCell>
											<div className="flex items-center justify-end gap-1">
												<GrantCardButton onClick={() => setGrantCardUser(user)} />
												<Button
													variant="ghost"
													size="icon-sm"
													title="Modifier"
													onClick={() => setEditUser(user)}
												>
													<Pencil />
												</Button>
												<Button
													variant="ghost"
													size="icon-sm"
													title="Supprimer"
													className="text-destructive hover:bg-destructive/10 hover:text-destructive"
													onClick={() => setDeleteUser(user)}
												>
													<Trash2 />
												</Button>
											</div>
										</TableCell>
									</TableRow>
								))}
							</TableBody>
						</Table>
					)}

					<div className="mt-4 flex items-center justify-between gap-2">
						<Button
							variant="outline"
							size="sm"
							disabled={page <= 0 || loading}
							onClick={() => gotoPage(page - 1)}
						>
							<ChevronLeft />
							Précédent
						</Button>
						<p className="text-muted-foreground text-sm">
							Page {page + 1} sur {totalPages}
						</p>
						<Button
							variant="outline"
							size="sm"
							disabled={page >= totalPages - 1 || loading}
							onClick={() => gotoPage(page + 1)}
						>
							Suivant
							<ChevronRight />
						</Button>
					</div>
				</CardContent>
			</Card>

			<EditUserDialog
				user={editUser}
				open={editUser !== null}
				onOpenChange={(open) => {
					if (!open) setEditUser(null);
				}}
				onSaved={() => loadUsers(page)}
			/>

			<DeleteUserDialog
				user={deleteUser}
				open={deleteUser !== null}
				onOpenChange={(open) => {
					if (!open) setDeleteUser(null);
				}}
				onDeleted={() => {
					if (users.length === 1 && page > 0) {
						setPage(page - 1);
					} else {
						loadUsers(page);
					}
				}}
			/>

			<CardsManagerDialog
				user={grantCardUser}
				open={grantCardUser !== null}
				onOpenChange={(open) => {
					if (!open) setGrantCardUser(null);
				}}
			/>
		</div>
	);
}
