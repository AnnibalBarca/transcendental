// Static front-only fixtures for the unused MoneyPackGrid (real-money
// packs concept, dropped — see the note in MoneyPackGrid.tsx). Nothing on
// the backend corresponds to these ids; `price` here is a display string
// in euros, unrelated to the in-game "wallet" currency used everywhere
// else in the shop (shop_catalog.price / users.wallet, both integers).
export interface MoneyPack {
	id: string;
	label: string;
	amount: string;
	price: string;
	accent: string;
	description?: string;
}

export const MONEY_PACKS: MoneyPack[] = [
	{
		id: "pack-1",
		label: "5 cacahouètes",
		amount: "Starter",
		price: "0,99 €",
		accent: "hot",
		description: "Petit pack d'essai pour lancer l'achat.",
	},
	{
		id: "pack-2",
		label: "25 cacahouètes",
		amount: "Classique",
		price: "2,99 €",
		accent: "warm",
		description: "Un point d'entrée délicieux pour la boutique ;)",
	},
	{
		id: "pack-3",
		label: "100 cacahouètes",
		amount: "Boost",
		price: "9,99 €",
		accent: "gold",
		description: "De quoi faire plusieurs achats.",
	},
	{
		id: "pack-4",
		label: "250 cacahouètes",
		amount: "Max",
		price: "19,99 €",
		accent: "deep",
		description: "Le pack le plus rentable !",
	},
];

