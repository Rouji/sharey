#include <stdbool.h>
#include <stdio.h>

typedef struct ShareyConfig ShareyConfig;
typedef struct ShareyUploader ShareyUploader;
typedef struct ShareyInput ShareyInput;

bool sharey_upload(ShareyUploader *uploader, ShareyConfig *config,
                   ShareyInput *input);
ShareyConfig *sharey_config_from_file(const char *path);
void sharey_config_free(ShareyConfig *config);
ShareyUploader *sharey_uploader_new();
void sharey_uploader_free(ShareyUploader *uploader);
ShareyInput *sharey_input_from_file(const char *path);
void sharey_input_free(ShareyInput *input);

int main(int argc, char **argv) {
  ShareyUploader *uploader = sharey_uploader_new();
  ShareyConfig *config = sharey_config_from_file(argv[1]);
  ShareyInput *input = sharey_input_from_file(argv[2]);
  sharey_upload(uploader, config, input);
  sharey_config_free(config);
  sharey_uploader_free(uploader);
  return 0;
}
