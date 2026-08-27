import { useCallback, useEffect, useState } from "react";
import { Check, Layers, Loader2, RefreshCw, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { CHESS_CARDS, RARITY_NAMES, maxRarityForCard, cardImageDeckUrl, type CardId } from "@/data/cards";
import { adminService } from "../services/adminService";
import type { AdminUser } from "../types";

interface CardsManagerDialogProps {
	user: AdminUser | null;
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

const CARD_IDS = (Object.keys(CHESS_CARDS) as CardId[]).filter((id) => id !== "18");

function RarityMenu({
	cardId,
	maxRarity,
	rarities,
	onToggle,
	pending,
	onClose,
}: {
	cardId: string;
	maxRarity: number;
	rarities: number[];
	onToggle: (rarity: number) => void;
	pending: boolean;
	onClose: () => void;
}) {
	return (
		<div
			className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
			onClick={onClose}
		>
			<div
				className="bg-popover text-popover-foreground w-56 rounded-xl border p-3 shadow-xl"
				onClick={(e) => e.stopPropagation()}
			>
				<div className="mb-2 flex items-start justify-between gap-2">
					<div>
						<p className="text-sm font-semibold leading-tight">
							{CHESS_CARDS[cardId as CardId]?.title}
						</p>
					</div>
					<button
						type="button"
						onClick={onClose}
						className="text-muted-foreground hover:text-foreground rounded-md p-1"
					>
						<X className="size-4" />
					</button>
				</div>
				<div className="flex flex-col gap-1">
					{[0, 1, 2]
						.filter((rarity) => rarity <= maxRarity)
						.map((rarity) => {
							const has = rarities.includes(rarity);
							const isPending = pending && !has;
							return (
								<button
									key={rarity}
									type="button"
									disabled={pending}
									onClick={() => onToggle(rarity)}
									className={`flex items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-sm transition-colors ${
										has
											? "border-primary/50 bg-primary/10 hover:bg-primary/20"
											: "border-border hover:bg-muted"
									}`}
								>
									<span>
										{RARITY_NAMES[rarity]}
										{rarity === 0 && <span className="text-muted-foreground ml-1 text-xs">(Commun)</span>}
									</span>
									{isPending ? (
										<Loader2 className="size-3.5 animate-spin" />
									) : has ? (
										<Check className="size-3.5 text-primary" />
									) : (
										<span className="text-muted-foreground text-xs">+</span>
									)}
								</button>
							);
						})}
				</div>
			</div>
		</div>
	);
}

function CardsManagerForm({ user, onClose }: { user: AdminUser; onClose: () => void }) {
	const [owned, setOwned] = useState<Record<string, number[]>>({});
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [pending, setPending] = useState<string | null>(null);
	const [openCard, setOpenCard] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const cards = await adminService.listPlayerCards(user.id);
			const map: Record<string, number[]> = {};
			for (const c of cards) {
				(map[c.card_id] ??= []).push(c.rarity);
			}
			for (const id of CARD_IDS) {
				map[id] ??= [];
			}
			setOwned(map);
		} catch (e) {
			setError(e instanceof Error ? e.message : "Impossible de charger les cartes");
		} finally {
			setLoading(false);
		}
	}, [user.id]);

	useEffect(() => {
		load();
	}, [load]);

	const toggleRarity = async (cardId: string, rarity: number) => {
		const has = owned[cardId]?.includes(rarity) ?? false;
		setPending(`${cardId}:${rarity}`);
		setError(null);
		try {
			if (has) {
				await adminService.removeCardRarity(user.id, cardId, rarity);
			} else {
				await adminService.grantCard(user.id, { card_id: cardId, rarity });
			}
			await load();
		} catch (e) {
			setError(e instanceof Error ? e.message : "Une erreur est survenue");
		} finally {
			setPending(null);
		}
	};

	const unlockedCount = CARD_IDS.filter((id) => (owned[id]?.length ?? 0) > 0).length;

	return (
		<>
			<DialogHeader className="gap-1">
				<DialogTitle>Cartes de {user.username || user.email}</DialogTitle>
				<DialogDescription>
					{unlockedCount}/{CARD_IDS.length}
				</DialogDescription>
			</DialogHeader>

			<div className="flex items-center justify-between gap-2">
				{error ? (
					<p className="text-destructive rounded-md bg-destructive/10 px-3 py-1.5 text-sm">
						{error}
					</p>
				) : (
					<p className="text-muted-foreground text-sm">

					</p>
				)}
				<Button variant="outline" size="sm" onClick={load} disabled={loading}>
					<RefreshCw className={loading ? "animate-spin" : ""} />
					Actualiser
				</Button>
			</div>

			{loading ? (
				<div className="flex flex-col items-center justify-center gap-2 py-16 text-muted-foreground">
					<Loader2 className="size-6 animate-spin" />
					<p className="text-sm">Chargement des cartes…</p>
				</div>
			) : (
				<div className="grid max-h-[62vh] grid-cols-3 gap-3 overflow-y-auto pr-1 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
					{CARD_IDS.map((id) => {
						const rarities = owned[id] ?? [];
						const unlocked = rarities.length > 0;
						const data = CHESS_CARDS[id];
						return (
							<div key={id} className="relative">
								<button
									type="button"
									onClick={() => setOpenCard(openCard === id ? null : id)}
									className={`group relative flex aspect-[3/4] w-full cursor-pointer flex-col overflow-hidden rounded-lg border text-left transition-all ${
										unlocked
											? "border-primary/40 bg-card shadow-sm hover:border-primary/70 hover:shadow-md"
											: "border-border/70 bg-muted/40 opacity-60 grayscale hover:opacity-80"
									}`}
								>
									<div
										className="absolute inset-0 bg-cover bg-center"
										style={{ backgroundImage: `url(${cardImageDeckUrl(id)})` }}
									/>
									<div
										className={`absolute inset-0 ${
											unlocked
												? "bg-gradient-to-t from-black/85 via-black/20 to-transparent"
												: "bg-gradient-to-t from-black/70 via-black/30 to-black/20"
										}`}
									/>
									<span className="bg-background/70 absolute top-1.5 left-1.5 rounded px-1 py-0.5 text-[9px] text-muted-foreground">
										#{id}
									</span>
									<div className="absolute right-0 bottom-0 left-0 p-1.5">
										<p className="line-clamp-2 text-[11px] leading-tight font-semibold text-white">
											{data.title}
										</p>
										<p className="text-white/70 text-[9px]">
										</p>
									</div>
								</button>

								{openCard === id && (
									<RarityMenu
										cardId={id}
										maxRarity={maxRarityForCard(id)}
										rarities={rarities}
										pending={pending !== null}
										onToggle={(rarity) => toggleRarity(id, rarity)}
										onClose={() => setOpenCard(null)}
									/>
								)}
							</div>
						);
					})}
				</div>
			)}

			<div className="mt-2 flex justify-end">
				<Button variant="outline" onClick={onClose}>
					Fermer
				</Button>
			</div>
		</>
	);
}

export function CardsManagerDialog({ user, open, onOpenChange }: CardsManagerDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[85vh] overflow-y-auto">
				{user && (
					<CardsManagerForm
						key={user.id}
						user={user}
						onClose={() => onOpenChange(false)}
					/>
				)}
			</DialogContent>
		</Dialog>
	);
}

export function GrantCardButton({ onClick }: { onClick: () => void }) {
	return (
		<Button variant="ghost" size="icon-sm" title="Gérer les cartes" onClick={onClick}>
			<Layers />
		</Button>
	);
}
