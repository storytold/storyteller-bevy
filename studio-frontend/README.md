# studio-frontend

This is the primary microfrontend to be embedded in the [Storyteller.ai
website](https://fakeyou.com/studio).

### Running

The project can be served locally using the `serve` target:
```sh
npx nx serve studio-frontend
```

### Configuring application startup

After running the `serve` command above, the web app can be viewed in the
browser at `http://localhost:4200` and configured through a set of URL
parameters which are documented [here](../studio-web/src/lib/url-params.ts).

For example, to load the bundled `sample-room.gltf` scene in `StudioMode.Editor`,
visit: http://localhost:4200/?mode=studio&objectId=sample-room.gltf
