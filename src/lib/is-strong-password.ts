export default function isStrongPassword(password: string): boolean {
	const hasUppercase = (/[A-Z]/u).test(password);
	const hasLowercase = (/[a-z]/u).test(password);
	const nasNumeric = (/[0-9]/u).test(password);
	const hasSpecial = (/[!@#$%^&*(),.?":{}|<>]/u).test(password);
	const hasValidLength = password.length >= 8 && password.length <= 100;

	return hasUppercase && hasLowercase && nasNumeric && hasSpecial && hasValidLength;
}
