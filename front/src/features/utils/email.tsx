import extractErrorMessage, { API_LOGIN } from "../auth/services/authService";

export async function sendVerificationEmail(): Promise<void> {
	const response = await fetch(`${API_LOGIN}/send_validation_email_code`, {
		method: "POST",
		credentials: "include",
	});
	if (!response.ok) {
		const msg = await extractErrorMessage(response, "Failed to send verification email");
		throw new Error(msg);
	}
}

export async function sendPasswordReset(email: string): Promise<void> {
	const response = await fetch(`${API_LOGIN}/forgot_password`, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ email }),
		credentials: "include",
	});
	if (!response.ok) {
		const msg = await extractErrorMessage(response, "Failed to send reset email");
		throw new Error(msg);
	}
}

export async function sendVerificationEmailProviderChange(email: string): Promise<void> {
	const response = await fetch(`${API_LOGIN}/change_provider/email/send_validation_code`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ email: email }),
		credentials: "include",
	});
	if (!response.ok) {
		const msg = await extractErrorMessage(response, "Failed to send verification email");
		throw new Error(msg);
	}
}
