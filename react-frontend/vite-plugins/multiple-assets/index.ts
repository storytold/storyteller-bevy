// FIXME: Oh my god, did Danny seriously just copy and paste this entire folder
//        from `studio-frontend` instead of creating a separate project for it
//        and importing it like a decent, law-abiding citizen? Wow, dude really
//        needs to get his shit together.

// NOTE: This is an inline fork of this npm package:
// https://github.com/nguyenbatranvan/vite-multiple-assets
//
// The published version at time of writing doesn't include mime types for many
// of the asset types Bevy needs to load.

import {PluginOption, ViteDevServer} from "vite";
import {ServerMiddleWare} from "./server";
import {buildMiddleWare} from "./build";

export default function DynamicPublicDirectory(assets: string[]): PluginOption {
    return {
        configureServer(server: ViteDevServer) {
            return ServerMiddleWare(server, assets)
        },
        async writeBundle(options) {
            buildMiddleWare(options, assets)
        },
        name: "dynamic assets"
    };
}
