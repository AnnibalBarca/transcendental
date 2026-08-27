import { useState } from "react";
import { Loader2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { adminService } from "../services/adminService";
import type { AdminUser } from "../types";

interface DeleteUserDialogProps {
	user: AdminUser | null;
	open: boolean;
	onOpenChange: (open: boolean) => void;
	onDeleted: () => void;
}

export function DeleteUserDialog({ user, open, onOpenChange, onDeleted }: DeleteUserDialogProps) {
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	if (!user) return null;

	const handleDelete = async () => {
		setSubmitting(true);
		setError(null);
		try {
			await adminService.deleteUser(user.id);
			onOpenChange(false);
			onDeleted();
		} catch (e) {
			setError(e instanceof Error ? e.message : "Une erreur est survenue");
		} finally {
			setSubmitting(false);
		}
	};

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<DialogHeader>
					<DialogTitle className="flex items-center gap-2">
						<AlertTriangle className="text-destructive size-5" />
						Supprimer l'utilisateur
					</DialogTitle>
					<DialogDescription>
						Voulez-vous vraiment supprimer le compte{" "}
						<strong>{user.username || user.email}</strong> ? Cette action est
						irréversible.
					</DialogDescription>
				</DialogHeader>

				{error && (
					<p className="text-destructive rounded-md bg-destructive/10 px-3 py-2 text-sm">
						{error}
					</p>
				)}

				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>
						Annuler
					</Button>
					<Button variant="destructive" onClick={handleDelete} disabled={submitting}>
						{submitting && <Loader2 className="animate-spin" />}
						Supprimer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
