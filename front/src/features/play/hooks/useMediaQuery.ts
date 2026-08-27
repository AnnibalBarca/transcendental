import { useSyncExternalStore } from "react";

function subscribe(query: string, callback: () => void) {
	const mql = window.matchMedia(query);
	mql.addEventListener("change", callback);
	return () => mql.removeEventListener("change", callback);
}

export function useMediaQuery(query: string): boolean {
	return useSyncExternalStore(
		(callback) => subscribe(query, callback),
		() => window.matchMedia(query).matches,
		() => false,
	);
}

export const DESKTOP_QUERY = "(min-width: 1024px)";

export function useIsDesktop(): boolean {
	return useMediaQuery(DESKTOP_QUERY);
}
