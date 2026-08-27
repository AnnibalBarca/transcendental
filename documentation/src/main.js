import { createApp, h } from "vue";
import { ApiReference } from "@scalar/api-reference";
import openApiSpec from "../openapi.json";

createApp(h(ApiReference, {
  configuration: {
    title: "ft_transcendence — API Reference",
    content: openApiSpec,
    darkMode: true,
    hideDownloadButton: false,
  },
})).mount("#app");
