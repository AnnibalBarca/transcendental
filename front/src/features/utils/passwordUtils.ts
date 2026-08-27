export const getPasswordStrengthLevel = (password: string): number => {
	let volume = 0;

	if (password.length === 0) return 0;

	if (/[A-Z]/.test(password)) volume += 26;
	if (/[a-z]/.test(password)) volume += 26;
	if (/\d/.test(password)) volume += 10;
	if (/[^A-Za-z0-9]/.test(password)) volume += 32;

	const strength = password.length * Math.log(volume);

	if (strength <= 0) return 0;
	if (strength < 16) return 1;
	if (strength < 32) return 2;
	if (strength < 48) return 3;
	if (strength < 64) return 4;
	return 5;
};
