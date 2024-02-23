export default function isValidEmail(email: string): { isValid: boolean; message: string } {
	const hasAt = email.includes("@");
	const hasDot = email.includes(".");
	const hasValidLength = email.length >= 3 && email.length <= 100;
	const isValid = hasAt && hasDot && hasValidLength;

	return {
		isValid,
		message: isValid ? "" : "Invalid email address",
	};
}
