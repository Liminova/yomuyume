export function validateEmail(email: string): boolean {
	return (/\S+@\S+\.\S+/u).test(email);
}
