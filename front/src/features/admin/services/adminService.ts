import { API_USER } from "@/api/api";
import type {
	AdminPermission,
	AdminRole,
	AdminRoute,
	AdminUser,
	UpdateUserPayload,
} from "../types";

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
	const res = await fetch(url, {
		credentials: "include",
		headers: {
			"Content-Type": "application/json",
			...(options?.headers ?? {}),
		},
		...options,
	});

	if (!res.ok) {
		let message = `HTTP ${res.status}`;
		try {
			const data = await res.json();
			if (typeof data?.error === "string" && data.error.trim()) {
				message = data.error;
			} else if (typeof data?.message === "string" && data.message.trim()) {
				message = data.message;
			}
		} catch {
			const text = await res.text().catch(() => "");
			if (text.trim()) message = text;
		}
		throw new Error(message);
	}

	return res.json() as Promise<T>;
}

export interface AdminUserPage {
	users: AdminUser[];
	total: number;
}

export const adminService = {
	async listUsers(offset: number, limit: number): Promise<AdminUserPage> {
		const data = await fetchJson<{
			status: number;
			users?: AdminUser[];
			total?: number;
		}>(`${API_USER}/admin/users`, {
			method: "POST",
			body: JSON.stringify({ offset, limit }),
		});
		return {
			users: data.users ?? [],
			total: data.total ?? 0,
		};
	},

	async updateUser(id: string, payload: UpdateUserPayload): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/users/${id}`, {
			method: "PATCH",
			body: JSON.stringify(payload),
		});
	},

	async deleteUser(id: string): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/users/${id}`, {
			method: "DELETE",
		});
	},

	async addUserRole(userId: string, roleId: number): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/users/${userId}/roles`, {
			method: "POST",
			body: JSON.stringify({ id: roleId }),
		});
	},

	async removeUserRole(userId: string, roleId: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/users/${userId}/roles/${roleId}`,
			{ method: "DELETE" },
		);
	},

	async listPlayerCards(id: string): Promise<{ card_id: string; rarity: number }[]> {
		const data = await fetchJson<{
			status: number;
			cards?: { card_id: string; rarity: number }[];
		}>(`${API_USER}/admin/users/${id}/cards`);
		return data.cards ?? [];
	},

	async grantCard(
		id: string,
		payload: { card_id: string; rarity?: number },
	): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/users/${id}/cards`, {
			method: "POST",
			body: JSON.stringify(payload),
		});
	},

	async removeCardRarity(id: string, cardId: string, rarity: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/users/${id}/cards/${cardId}/${rarity}`,
			{ method: "DELETE" },
		);
	},

	async listRoles(): Promise<AdminRole[]> {
		const data = await fetchJson<{ status: number; roles?: AdminRole[] }>(
			`${API_USER}/admin/roles`,
		);
		return data.roles ?? [];
	},

	async createRole(payload: {
		name: string;
		description: string;
	}): Promise<AdminRole> {
		const data = await fetchJson<{ status: number; role?: AdminRole }>(
			`${API_USER}/admin/roles`,
			{
				method: "POST",
				body: JSON.stringify(payload),
			},
		);
		return data.role!;
	},

	async updateRole(
		id: number,
		payload: { name: string; description: string },
	): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/roles/${id}`, {
			method: "PATCH",
			body: JSON.stringify(payload),
		});
	},

	async deleteRole(id: number): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/roles/${id}`, {
			method: "DELETE",
		});
	},

	async addRolePermission(roleId: number, permissionId: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/roles/${roleId}/permissions`,
			{
				method: "POST",
				body: JSON.stringify({ id: permissionId }),
			},
		);
	},

	async removeRolePermission(roleId: number, permissionId: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/roles/${roleId}/permissions/${permissionId}`,
			{ method: "DELETE" },
		);
	},

	async listPermissions(): Promise<AdminPermission[]> {
		const data = await fetchJson<{
			status: number;
			permissions?: AdminPermission[];
		}>(`${API_USER}/admin/permissions`);
		return data.permissions ?? [];
	},

	async createPermission(payload: {
		name: string;
		description: string;
	}): Promise<AdminPermission> {
		const data = await fetchJson<{ status: number; permission?: AdminPermission }>(
			`${API_USER}/admin/permissions`,
			{
				method: "POST",
				body: JSON.stringify(payload),
			},
		);
		return data.permission!;
	},

	async updatePermission(
		id: number,
		payload: { name: string; description: string },
	): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/permissions/${id}`, {
			method: "PATCH",
			body: JSON.stringify(payload),
		});
	},

	async deletePermission(id: number): Promise<void> {
		await fetchJson<{ status: number }>(`${API_USER}/admin/permissions/${id}`, {
			method: "DELETE",
		});
	},

	async addPermissionRoute(permissionId: number, routeId: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/permissions/${permissionId}/routes`,
			{
				method: "POST",
				body: JSON.stringify({ id: routeId }),
			},
		);
	},

	async removePermissionRoute(permissionId: number, routeId: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/permissions/${permissionId}/routes/${routeId}`,
			{ method: "DELETE" },
		);
	},

	async listRoutes(): Promise<AdminRoute[]> {
		const data = await fetchJson<{ status: number; routes?: AdminRoute[] }>(
			`${API_USER}/admin/routes`,
		);
		return data.routes ?? [];
	},

	async setRouteRateLimit(routeId: number, requestsPerMinute: number): Promise<void> {
		await fetchJson<{ status: number }>(
			`${API_USER}/admin/routes/${routeId}/rate-limit`,
			{
				method: "PUT",
				body: JSON.stringify({ requests_per_minute: requestsPerMinute }),
			},
		);
	},
};
