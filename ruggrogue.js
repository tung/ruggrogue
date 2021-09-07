mergeInto(LibraryManager.library, {
    'ruggrogue_sync_idbfs': function () {
        FS.syncfs(false, function (err) {});
    },
});
