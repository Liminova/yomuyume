export default function isStrongPassword(password: string): { isStrong: boolean; message: string } {
	const hasUppercase = /[A-Z]/u.test(password);
	const hasLowercase = /[a-z]/u.test(password);
	const nasNumeric = /[0-9]/u.test(password);
	const hasSpecial = /[!@#$%^&*(),.?":{}|<>]/u.test(password);
	const hasValidLength = password.length >= 8 && password.length <= 100;
	const isStrong = hasUppercase && hasLowercase && nasNumeric && hasSpecial && hasValidLength;

	return {
		isStrong,
		message: isStrong
			? ""
			: "Password must contain at least one uppercase letter, one lowercase letter, one number, one special character, and be at least 8 characters long",
	};
}
