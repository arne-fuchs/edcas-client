pub fn get_increase_texture_resolution_preset() -> String {
    String::from(
        "\
<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<GraphicsConfig>
	<Planets>
		<Ultra>
			<TextureSize>4096</TextureSize>
			<WorkPerFrame>512</WorkPerFrame>
		</Ultra>
	</Planets>
	<GalaxyBackground>
		<High>
			<TextureSize>4096</TextureSize>
		</High>
	</GalaxyBackground>
	<Envmap>
		<High>
			<TextureSize>1024</TextureSize>
			<NumMips>10</NumMips>
		</High>
	</Envmap>[2]
</GraphicsConfig>
"
    )
}

pub fn get_8gb_plus_preset() -> String {
    String::from(
        "\
<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<GraphicsConfig>
  <GalaxyMap>
	<High>
		<LocalisationName>$QUALITY_HIGH;</LocalisationName>
		<NebulasCount>100</NebulasCount>
		<NebulasInBackgroundCount>100</NebulasInBackgroundCount>
		<LowResNebulasCount>50</LowResNebulasCount>
		<HighResNebulasCount>10</HighResNebulasCount>
		<LowResNebulaDimensions>64</LowResNebulaDimensions>
		<HighResNebulaDimensions>256</HighResNebulaDimensions>
		<LowResSamplesCount>276</LowResSamplesCount>
		<HighResSamplesCount>552</HighResSamplesCount>
		<MilkyWayInstancesCount>16000</MilkyWayInstancesCount>
		<LocalDustBrightness>0.05</LocalDustBrightness>
		<MilkywayInstancesBrightness>1.0</MilkywayInstancesBrightness>
		<MilkywayInstancesSize>1.0</MilkywayInstancesSize>
		<MilkyWayInstancesOffscreenRTEnabled>false</MilkyWayInstancesOffscreenRTEnabled>
		<StarInstanceCount>20000</StarInstanceCount>
	</High>
  </GalaxyMap>
  <GalaxyBackground>
	<High>
	<LocalisationName>$QUALITY_HIGH;</LocalisationName>
	<TextureSize>4096</TextureSize>
	</High>
  </GalaxyBackground>
  <Planets>
    <Ultra>
      <LocalisationName>$QUALITY_ULTRA;</LocalisationName>
      <TextureSize>4096</TextureSize>
      <AtmosphereSteps>6</AtmosphereSteps>
      <CloudsEnabled>true</CloudsEnabled>
      <WorkPerFrame>328</WorkPerFrame>
	  <TexturePoolBudget>100</TexturePoolBudget>
    </Ultra>
  </Planets>
  <Envmap>
	<High>
		<LocalisationName>$QUALITY_HIGH;</LocalisationName>
		<TextureSize>512</TextureSize>
		<NumMips>16</NumMips>
	</High>
  </Envmap>
</GraphicsConfig>
"
    )
}

pub fn get_increased_star_count_preset() -> String {
    String::from(
        "\
<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<GraphicsConfig>
	<!-- Extreme increase to StarInstanceCount -->
	<GalaxyMap>
		<Low>
			<LocalisationName>$QUALITY_LOW; (40B StarInstanceCount)</LocalisationName>
			<StarInstanceCount>40000000</StarInstanceCount>
		</Low>
		<Medium>
			<LocalisationName>$QUALITY_MEDIUM; (40B StarInstanceCount)</LocalisationName>
			<StarInstanceCount>40000000</StarInstanceCount>
		</Medium>
		<High>
			<LocalisationName>$QUALITY_HIGH; (40B StarInstanceCount)</LocalisationName>
			<StarInstanceCount>40000000</StarInstanceCount>
		</High>
	</GalaxyMap>
</GraphicsConfig>
"
    )
}

pub fn get_better_skybox_preset() -> String {
    String::from(
        "\
<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<GraphicsConfig>
	<!-- Extreme increase to StarInstanceCount -->
	<GalaxyMap>
		<Low>
			<LocalisationName>$QUALITY_LOW; (40B StarInstanceCount)</LocalisationName>
			<StarInstanceCount>40000000</StarInstanceCount>
		</Low>
		<Medium>
			<LocalisationName>$QUALITY_MEDIUM; (40B StarInstanceCount)</LocalisationName>
			<StarInstanceCount>40000000</StarInstanceCount>
            <LowResNebulasCount>20</LowResNebulasCount>
            <HighResNebulasCount>8</HighResNebulasCount>
            <NebulasCount>50</NebulasCount>
		    <NebulasInBackgroundCount>50</NebulasInBackgroundCount>
		</Medium>
		<High>
			<LocalisationName>$QUALITY_HIGH; (40B StarInstanceCount)</LocalisationName>
			<StarInstanceCount>40000000</StarInstanceCount>
            <LowResNebulasCount>25</LowResNebulasCount>
            <HighResNebulasCount>10</HighResNebulasCount>
            <NebulasCount>100</NebulasCount>
		    <NebulasInBackgroundCount>100</NebulasInBackgroundCount>
		</High>
	</GalaxyMap>
    <GalaxyBackground>
		<Medium>
			<LocalisationName>$QUALITY_MEDIUM; (2K)</LocalisationName>
			<TextureSize>2048</TextureSize>
		</Medium>
		<High>
			<LocalisationName>$QUALITY_HIGH; (4K)</LocalisationName>
			<TextureSize>4096</TextureSize>
		</High>
	</GalaxyBackground>
    <Environment>
		<Ultra>
			<LocalisationName>$QUALITY_ULTRA; (GalaxyBackground Quality 3)</LocalisationName>
			<Item>
				<Feature>GalaxyBackground</Feature>
				<QualitySetting>3</QualitySetting>
			</Item>
		</Ultra>
	</Environment>
</GraphicsConfig>
"
    )
}