import { useCallback, useEffect, useState } from "react";
import { Check, Gauge, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { adminService } from "../services/adminService";
import type { AdminRoute } from "../types";

export function RateLimitsPanel() {
	const [routes, setRoutes] = useState<AdminRoute[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [drafts, setDrafts] = useState<Record<number, string>>({});
	const [savingId, setSavingId] = useState<number | null>(null);
	const [saveError, setSaveError] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const data = await adminService.listRoutes();
			setRoutes(data);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Impossible de charger les routes");
			setRoutes([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		let cancelled = false;
		(async () => {
			try {
				const data = await adminService.listRoutes();
				if (cancelled) return;
				setRoutes(data);
				setError(null);
			} catch (e) {
				if (cancelled) return;
				setError(e instanceof Error ? e.message : "Impossible de charger les routes");
				setRoutes([]);
			} finally {
				if (!cancelled) setLoading(false);
			}
		})();
		return () => {
			cancelled = true;
		};
	}, []);

	const setDraft = (routeId: number, value: string) => {
		setDrafts((prev) => ({ ...prev, [routeId]: value }));
	};

	const save = async (route: AdminRoute) => {
		const raw = drafts[route.id] ?? String(route.requests_per_minute ?? "");
		const value = Number(raw);
		if (Number.isNaN(value) || value < 0) return;

		setSavingId(route.id);
		setSaveError(null);
		try {
			await adminService.setRouteRateLimit(route.id, Math.round(value));
			setRoutes((prev) =>
				prev.map((r) =>
					r.id === route.id ? { ...r, requests_per_minute: Math.round(value) } : r,
				),
			);
			setDrafts((prev) => {
				const next = { ...prev };
				delete next[route.id];
				return next;
			});
		} catch (e) {
			setSaveError(e instanceof Error ? e.message : "Impossible d'enregistrer");
		} finally {
			setSavingId(null);
		}
	};

	return (
		<div className="flex flex-col gap-4">
			<div className="flex items-center justify-between gap-2">
				<p className="text-muted-foreground text-sm">
					{routes.length} route{routes.length > 1 ? "s" : ""} API · limite par minute
				</p>
				<Button size="sm" variant="outline" onClick={load} disabled={loading}>
					<Gauge className={loading ? "animate-spin" : ""} />
					Rafraîchir
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

			{saveError && (
				<p className="text-destructive rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-2 text-sm">
					{saveError}
				</p>
			)}

			{loading ? (
				<div className="flex flex-col items-center justify-center gap-2 py-16 text-muted-foreground">
					<Loader2 className="size-6 animate-spin" />
					<p className="text-sm">Chargement des routes…</p>
				</div>
			) : (
				<div className="bg-card rounded-xl border shadow-sm">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead>Méthode</TableHead>
								<TableHead>Route</TableHead>
								<TableHead>Nom</TableHead>
								<TableHead className="text-right">Requêtes / min</TableHead>
								<TableHead className="w-24" />
							</TableRow>
						</TableHeader>
						<TableBody>
							{routes.length === 0 ? (
								<TableRow>
									<TableCell colSpan={5} className="text-muted-foreground py-12 text-center">
										Aucune route trouvée
									</TableCell>
								</TableRow>
							) : (
								routes.map((route) => {
									const value =
										drafts[route.id] ?? String(route.requests_per_minute ?? "");
									const dirty = value !== String(route.requests_per_minute ?? "");
									return (
										<TableRow key={route.id}>
											<TableCell>
												<Badge variant="outline">{route.method}</Badge>
											</TableCell>
											<TableCell>
												<p className="font-mono text-xs">{route.path}</p>
											</TableCell>
											<TableCell>
												<p className="text-muted-foreground truncate text-xs">
													{route.name || "—"}
												</p>
											</TableCell>
											<TableCell className="text-right">
												<Input
													type="number"
													min="0"
													value={value}
													onChange={(e) => setDraft(route.id, e.target.value)}
													className="ml-auto w-24 text-right"
												/>
											</TableCell>
											<TableCell>
												<div className="flex justify-end">
													<Button
														size="sm"
														disabled={!dirty || savingId === route.id}
														onClick={() => save(route)}
													>
														{savingId === route.id ? (
															<Loader2 className="animate-spin" />
														) : (
															<Check />
														)}
														Enregistrer
													</Button>
												</div>
											</TableCell>
										</TableRow>
									);
								})
							)}
						</TableBody>
					</Table>
				</div>
			)}
		</div>
	);
}
