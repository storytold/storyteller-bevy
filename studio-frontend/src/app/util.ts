export function isDevEnvironment(): boolean {
	return window.location.hostname === "localhost"
		|| window.location.hostname === "127.0.0.1"
		|| window.location.hostname.includes("deploy-preview");
}
