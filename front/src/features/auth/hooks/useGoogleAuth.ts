import { useEffect, useRef, useCallback } from "react";

const GOOGLE_CLIENT_ID =
	"348021368709-cbuqeuhbf617rqf4i6um39koco81smh5.apps.googleusercontent.com";

interface GoogleCodeClientConfig {
	client_id: string;
	scope: string;
	ux_mode: "popup" | "redirect";
	redirect_uri?: string;
	callback: (response: GoogleCodeResponse) => void;
}

interface GoogleCodeResponse {
	code?: string;
	error?: string;
	error_description?: string;
}

interface GoogleCodeClient {
	requestCode: () => void;
}

declare global {
	interface Window {
		google?: {
			accounts: {
				oauth2: {
					initCodeClient: (config: GoogleCodeClientConfig) => GoogleCodeClient;
				};
			};
		};
	}
}

export function useGoogleAuth(onSuccess: (code: string) => Promise<void>) {
	const googleClientRef = useRef<GoogleCodeClient | null>(null);

	useEffect(() => {
		const existing = document.getElementById("google-gsi-script");
		if (!existing) {
			const script = document.createElement("script");
			script.id = "google-gsi-script";
			script.src = "https://accounts.google.com/gsi/client";
			script.async = true;
			script.defer = true;
			document.body.appendChild(script);
		}
	}, []);

	const initGoogleClient = useCallback(() => {
		if (!window.google?.accounts?.oauth2 || googleClientRef.current) return;

		googleClientRef.current = window.google.accounts.oauth2.initCodeClient({
			client_id: GOOGLE_CLIENT_ID,
			scope: "openid email profile",
			ux_mode: "popup",
			redirect_uri: "postmessage",
			callback: async (response: GoogleCodeResponse) => {
				if (response.error) {
					alert("Google login failed: " + response.error);
					return;
				}
				if (!response.code) {
					alert("Google login failed: no code received");
					return;
				}
				try {
					await onSuccess(response.code);
				} catch (e: unknown) {
					const message = e instanceof Error ? e.message : "Google login failed";
					alert(message);
				}
			},
		});
	}, [onSuccess]);

	useEffect(() => {
		if (window.google?.accounts?.oauth2) {
			initGoogleClient();
		} else {
			const interval = setInterval(() => {
				if (window.google?.accounts?.oauth2) {
					initGoogleClient();
					if (googleClientRef.current) clearInterval(interval);
				}
			}, 200);
			setTimeout(() => clearInterval(interval), 10000);
		}
	}, [initGoogleClient]);

	const requestCode = useCallback(() => {
		if (googleClientRef.current) {
			googleClientRef.current.requestCode();
		} else {
			alert("Google script not loaded yet, please wait and retry.");
		}
	}, []);

	return { requestCode };
}
