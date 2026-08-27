export interface AdminUser {
	id: string;
	username: string | null;
	email: string;
	account_validated: boolean;
	email_validated: boolean;
	auth_provider: string;
	roles: string[];
	is_banned: boolean;
	wallet: number;
	ranked_elo: number;
	level: number;
	xp: number;
	picture_id: string;
}

export interface UpdateUserPayload {
	username?: string;
	email?: string;
	account_validated?: boolean;
	email_validated?: boolean;
	is_banned?: boolean;
	wallet?: number;
	ranked_elo?: number;
	xp?: number;
}

export interface AdminRole {
	id: number;
	name: string;
	description: string;
	permissions: string[];
}

export interface AdminPermission {
	id: number;
	name: string;
	description: string;
	routes: string[];
}

export interface AdminRoute {
	id: number;
	method: string;
	path: string;
	name: string;
	description: string;
	requests_per_minute: number | null;
}

export type AdminSection = "users" | "roles" | "permissions" | "rate-limits";
