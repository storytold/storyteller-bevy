#!/bin/bash

# Update this to a recent scene to download it.
scene_url="https://storage.googleapis.com/vocodes-public/media/d/2/d/s/e/d2dseec97gw06fk7mdavmk4pv87dk0dt/scene_d2dseec97gw06fk7mdavmk4pv87dk0dt.scn.ron"

filename="downloaded_scene.scn.ron"
directory="studio/assets/"
full_path="${directory}${filename}"

if [ ! -f "${full_path}" ]; then
  echo "Downloading ${scene_url}"
  mkdir -p "${directory}"
  wget "${scene_url}" -O "${full_path}"
fi

cargo run -p studio \
        --bin studio-headless \
        --features headless \
        -- \
        scene \
        -o frames/ \
        --frames 100 \
        --skybox "00ff00" \
				$filename

