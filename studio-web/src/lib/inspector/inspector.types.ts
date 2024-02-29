export type InspectorComponent
	= BevyReflect<"bevy_transform::components::transform::Transform", BevyTransform>
	| BevyReflect<"bevy_core::name::Name", BevyName>
	;

export interface BevyReflect<K extends string, T> {
	name: K;
	value: T;
}

export interface BevyTransform {
	rotation: Quat;
	scale: Vec3;
	translation: Vec3;
}

export interface BevyName {
	hash: number;
	name: string;
}

export interface Quat {
	w: number;
	x: number;
	y: number;
	z: number;
}

export interface Vec3 {
	x: number;
	y: number;
	z: number;
}
